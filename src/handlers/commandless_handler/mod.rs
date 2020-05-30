mod github_search;
mod link_url;
mod spellcheck;
mod unit_conversion;

use github_search::github_search;
use link_url::link_url;
use spellcheck::spellcheck;
use unit_conversion::unit_conversion;

use crate::config::{Config, Storage};
use crate::helpers::{check_format, BotResponse};
use crate::regex::{GITHUB_SEARCH, LINK_URL, UNIT_CONVERSION};

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
                let mut response = BotResponse::default();
                if UNIT_CONVERSION.is_match(&text.body)
                    && config.enable_unit_conversions
                    && text.relates_to.is_none()
                {
                    debug!("Entering commandless unit conversion path");
                    unit_conversion(&text, &config, &mut response);
                }
                if GITHUB_SEARCH.is_match(&text.body)
                    && !config.repos.is_empty()
                    && text.relates_to.is_none()
                {
                    debug!("Entering commandless github search path");
                    github_search(&text, &config, &api_client, &mut response).await;
                }
                if LINK_URL.is_match(&text.body)
                    && !config.links.is_empty()
                    && !config.linkers.is_empty()
                    && text.relates_to.is_none()
                {
                    debug!("Entering commandless url linking path");
                    link_url(&text, &config, &mut response);
                }
                let response = response;
                if response.is_some() {
                    send_notice(response.to_string(), &room_id, &client, &mut storage).await;
                } else if config.enable_corrections
                    && text.relates_to.is_none()
                    && storage.correction_time_cooldown(room_id)
                    && !config.correction_exclusion.contains(room_id)
                {
                    match spellcheck(text, sender, config) {
                        Some(v) => send_correction(v, &room_id, &client, &mut storage).await,
                        None => (),
                    }
                }
            }
            Err(e) => {
                error!("{:?}", e);
            }
        }
    }
}

async fn send_notice(
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
