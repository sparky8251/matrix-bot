use crate::config::{Config, SpellCheckKind, Storage};

use std::time::SystemTime;

use log::error;
use ruma_client::{
    api::r0::message::create_message_event,
    events::{
        room::message::{MessageEventContent, TextMessageEventContent},
        EventJson, EventType,
    },
    identifiers::{RoomId, UserId},
    HttpsClient,
};

pub async fn spellcheck(
    text: &TextMessageEventContent,
    sender: &UserId,
    room_id: &RoomId,
    client: &HttpsClient,
    config: &Config,
    storage: &mut Storage,
) {
    for i in config.incorrect_spellings.iter() {
        let mut body = String::new();
        match i {
            SpellCheckKind::SpellCheckInsensitive(v) => {
                if text.body.contains(&v.to_string().to_lowercase()) {
                    body = config
                        .correction_text
                        .replacen("{}", sender.localpart(), 1)
                        .replacen("{}", &v.to_string(), 1);
                }
            }
            SpellCheckKind::SpellCheckSensitive(v) => {
                if text.body.contains(&v.to_string()) {
                    body = config
                        .correction_text
                        .replacen("{}", sender.localpart(), 1)
                        .replacen("{}", &v.to_string(), 1);
                }
            }
        }
        if body != "" {
            let response = client
                .request(create_message_event::Request {
                    room_id: room_id.clone(), //FIXME: There has to be a better way than to clone here
                    event_type: EventType::RoomMessage,
                    txn_id: storage.next_txn_id(),
                    data: EventJson::from(MessageEventContent::Text(TextMessageEventContent {
                        body,
                        format: None,
                        formatted_body: None,
                        relates_to: None,
                    })),
                })
                .await;
            match response {
                Ok(_) => {
                    storage
                        .last_correction_time
                        .insert(room_id.clone(), SystemTime::now());
                    break; // only allow 1 match per event
                }
                Err(e) => {
                    error!("{:?}", e);
                }
            }
        }
    }
}
