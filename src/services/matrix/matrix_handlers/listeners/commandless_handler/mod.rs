//! Contains handlers and response functions for text based non-command events

mod github_search;
mod group_ping;
mod link_url;
mod spellcheck;
mod text_expansion;
mod unit_conversion;

use crate::config::MatrixListenerConfig;
use crate::helpers::{check_format, MatrixFormattedTextResponse, MatrixNoticeResponse};
use crate::messages::{MatrixMessage, MatrixMessageType};
use crate::regex::{GITHUB_SEARCH, GROUP_PING, LINK_URL, TEXT_EXPANSION, UNIT_CONVERSION};
use anyhow::anyhow;
use github_search::github_search;
use group_ping::group_ping;
use link_url::link_url;
use ruma::{
    events::room::message::{Relation, RoomMessageEventContent, TextMessageEventContent},
    RoomId, UserId,
};
use spellcheck::spellcheck;
use sqlx::{Pool, Sqlite};
use std::convert::TryInto;
use std::time::SystemTime;
use text_expansion::text_expansion;
use tokio::sync::mpsc::Sender;
use tracing::{debug, error, trace};
use unit_conversion::unit_conversion;

/// Handler for all text based non-command events
#[allow(clippy::too_many_arguments)]
pub async fn commandless_handler(
    text: &TextMessageEventContent,
    relates_to: Option<&Relation>,
    sender: &UserId,
    room_id: &RoomId,
    storage: &mut Pool<Sqlite>,
    config: &MatrixListenerConfig,
    api_client: &reqwest::Client,
    send: &mut Sender<MatrixMessage>,
) -> anyhow::Result<()> {
    if sender == config.mx_uname {
        // do nothing if message is from self
        trace!("Message is from self, doing nothing");
    } else {
        match check_format(text.formatted.as_ref().map(|f| &f.format)) {
            Ok(_) => {
                let mut notice_response = MatrixNoticeResponse::default();
                let mut text_response = MatrixFormattedTextResponse::default();
                if UNIT_CONVERSION.is_match(&text.body) && config.enable_unit_conversions {
                    debug!("Entering commandless unit conversion path");
                    unit_conversion(text, config, &mut notice_response);
                }
                if GITHUB_SEARCH.is_match(&text.body) && !config.repos.is_empty() {
                    debug!("Entering commandless github search path");
                    github_search(text, config, api_client, &mut notice_response).await;
                }
                if LINK_URL.is_match(&text.body)
                    && !config.links.is_empty()
                    && !config.linkers.is_empty()
                {
                    debug!("Entering commandless url linking path");
                    link_url(text, config, &mut notice_response)?;
                }
                if GROUP_PING.is_match(&text.body) {
                    debug!("Entering commandless group ping path");
                    group_ping(text, sender, config, &mut text_response);
                }
                if TEXT_EXPANSION.is_match(&text.body) {
                    debug!("Entering commandless text expansion path");
                    text_expansion(text, config, &mut notice_response);
                }

                let notice_response = notice_response;
                let text_response = text_response;

                if notice_response.is_some()
                    && send
                        .send(MatrixMessage {
                            room_id: Some(room_id.to_owned()),
                            message: MatrixMessageType::Response(
                                RoomMessageEventContent::notice_plain(notice_response.to_string()),
                            ),
                        })
                        .await
                        .is_err()
                {
                    return Err(anyhow!("Channel closed. Unable to send message."))?;
                }

                if text_response.is_some() {
                    let formatted_text = text_response.format_text().unwrap();
                    if send
                        .send(MatrixMessage {
                            room_id: Some(room_id.to_owned()),
                            message: MatrixMessageType::Response(
                                RoomMessageEventContent::text_html(
                                    text_response.to_string(),
                                    formatted_text,
                                ),
                            ),
                        })
                        .await
                        .is_err()
                    {
                        return Err(anyhow!("Channel closed. Unable to send message."))?;
                    }
                }
                if config.enable_corrections
                    && relates_to.is_none()
                    && correction_time_cooldown(&storage, room_id)
                    && !config.correction_exclusion.contains(room_id)
                    && !notice_response.is_some()
                    && !text_response.is_some()
                {
                    if let Some(v) = spellcheck(text, sender, config) {
                        match send
                            .send(MatrixMessage {
                                room_id: Some(room_id.to_owned()),
                                message: MatrixMessageType::Response(
                                    RoomMessageEventContent::text_plain(v),
                                ),
                            })
                            .await
                        {
                            Ok(_) => {
                                let _ = storage.insert(
                                    "last_correction_time_".to_owned() + &room_id.to_string(),
                                    SystemTime::now()
                                        .duration_since(SystemTime::UNIX_EPOCH)?
                                        .as_secs()
                                        .to_be_bytes()
                                        .to_vec(),
                                );
                                storage.flush()?;
                            }
                            Err(_) => Err(anyhow!("Channel closed. Unable to send message."))?,
                        };
                    }
                }
            }
            Err(e) => {
                Err(anyhow!("{}", e))?;
            }
        }
    }
    Ok(())
}

fn correction_time_cooldown(storage: &Pool<Sqlite>, room_id: &RoomId) -> bool {
    match storage.get("last_correction_time_".to_owned() + &room_id.to_string()) {
        Ok(t) => {
            match t {
                Some(v) => {
                    let bytes: [u8; 8] = v.to_vec().try_into().unwrap();
                    let old_time = u64::from_be_bytes(bytes);
                    let new_time = SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_secs();

                    if new_time < old_time + 300 {
                        true
                    } else {
                        false
                    }
                }
                None => true, // Will only be None if this client has not yet corrected anyone in specified room, so return true to allow correction
            }
        }
        Err(e) => {
            error!("Somehow unable to retrieve correction time cooldown key from database. Error is {}", e);
            false // Will only be Err in truly extreme situations. Log + return false to prevent correction and thus potential spam.
        }
    }
}
