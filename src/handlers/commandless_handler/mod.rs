//! Contains handlers and response functions for text based non-command events

mod github_search;
mod group_ping;
mod link_url;
mod spellcheck;
mod unit_conversion;

use github_search::github_search;
use group_ping::group_ping;
use link_url::link_url;
use spellcheck::spellcheck;
use unit_conversion::unit_conversion;

use crate::config::{Config, Storage};
use crate::helpers::{check_format, BotResponseNotice, BotResponseText};
use crate::regex::{GITHUB_SEARCH, GROUP_PING, LINK_URL, UNIT_CONVERSION};

use std::time::SystemTime;

use log::{debug, error, trace};
use ruma_client::{
    api::r0::message::create_message_event,
    events::{
        room::message::{MessageEventContent, NoticeMessageEventContent, TextMessageEventContent},
        EventJson, EventType,
    },
    identifiers::{RoomId, UserId},
    HttpsClient,
};

/// Handler for all text based non-command events
pub(super) async fn commandless_handler(
    text: &TextMessageEventContent,
    sender: &UserId,
    room_id: &RoomId,
    client: &HttpsClient,
    mut storage: &mut Storage,
    config: &Config,
    api_client: &reqwest::Client,
) {
    if sender == &config.mx_uname {
        // do nothing if message is from self
        trace!("Message is from self, doing nothing");
    } else {
        match check_format(&text.format) {
            Ok(_) => {
                let mut notice_response = BotResponseNotice::default();
                let mut text_response = BotResponseText::default();
                if UNIT_CONVERSION.is_match(&text.body)
                    && config.enable_unit_conversions
                    && text.relates_to.is_none()
                {
                    debug!("Entering commandless unit conversion path");
                    unit_conversion(&text, &config, &mut notice_response);
                }
                if GITHUB_SEARCH.is_match(&text.body)
                    && !config.repos.is_empty()
                    && text.relates_to.is_none()
                {
                    debug!("Entering commandless github search path");
                    github_search(&text, &config, &api_client, &mut notice_response).await;
                }
                if LINK_URL.is_match(&text.body)
                    && !config.links.is_empty()
                    && !config.linkers.is_empty()
                    && text.relates_to.is_none()
                {
                    debug!("Entering commandless url linking path");
                    link_url(&text, &config, &mut notice_response);
                }
                if GROUP_PING.is_match(&text.body) && text.relates_to.is_none() {
                    debug!("Entering commandless group ping path");
                    group_ping(&text, &sender, &config, &mut text_response);
                }

                let notice_response = notice_response;
                let text_response = text_response;

                if notice_response.is_some() {
                    send_notice(&notice_response, &room_id, &client, &mut storage).await;
                }
                if text_response.is_some() {
                    send_text(&text_response, &room_id, &client, &mut storage).await;
                }
                if config.enable_corrections
                    && text.relates_to.is_none()
                    && storage.correction_time_cooldown(room_id)
                    && !config.correction_exclusion.contains(room_id)
                    && !notice_response.is_some()
                    && !text_response.is_some()
                {
                    if let Some(v) = spellcheck(text, sender, config) {
                        send_correction(v, &room_id, &client, &mut storage).await
                    }
                }
            }
            Err(e) => {
                error!("{:?}", e);
            }
        }
    }
}

/// Sends a notice message containing the provided response body
async fn send_notice(
    notice_response: &BotResponseNotice,
    room_id: &RoomId,
    client: &HttpsClient,
    storage: &mut Storage,
) {
    let response = notice_response.to_string();
    match client
        .request(create_message_event::Request {
            room_id: room_id.clone(), //FIXME: There has to be a better way than to clone here
            event_type: EventType::RoomMessage,
            txn_id: storage.next_txn_id(),
            data: EventJson::from(MessageEventContent::Notice(NoticeMessageEventContent {
                body: response,
                format: None,
                formatted_body: None,
                relates_to: None,
            }))
            .into_json(),
        })
        .await
    {
        Ok(_) => (),
        Err(e) => error!("Unable to send response due to error {:?}", e),
    }
}

/// Sends a text message containing the provided response body
async fn send_text(
    text_response: &BotResponseText,
    room_id: &RoomId,
    client: &HttpsClient,
    storage: &mut Storage,
) {
    let response = text_response.to_string();
    match client
        .request(create_message_event::Request {
            room_id: room_id.clone(), //FIXME: There has to be a better way than to clone here
            event_type: EventType::RoomMessage,
            txn_id: storage.next_txn_id(),
            data: EventJson::from(MessageEventContent::Text(TextMessageEventContent {
                body: response,
                format: None,
                formatted_body: None,
                relates_to: None,
            }))
            .into_json(),
        })
        .await
    {
        Ok(_) => (),
        Err(e) => error!("Unable to send response due to error {:?}", e),
    }
}

/// Sends a text message and updates the last_correction_time on success
async fn send_correction(
    response: String,
    room_id: &RoomId,
    client: &HttpsClient,
    storage: &mut Storage,
) {
    match client
        .request(create_message_event::Request {
            room_id: room_id.clone(), //FIXME: There has to be a better way than to clone here
            event_type: EventType::RoomMessage,
            txn_id: storage.next_txn_id(),
            data: EventJson::from(MessageEventContent::Text(TextMessageEventContent {
                body: response,
                format: None,
                formatted_body: None,
                relates_to: None,
            }))
            .into_json(),
        })
        .await
    {
        Ok(_) => {
            storage
                .last_correction_time
                .insert(room_id.clone(), SystemTime::now());
        }
        Err(e) => error!("Unable to send response due to error {:?}", e),
    }
}
