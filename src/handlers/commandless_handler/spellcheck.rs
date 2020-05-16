use crate::{Config, Storage};

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
    sender: &UserId,
    room_id: &RoomId,
    client: &HttpsClient,
    config: &Config,
    storage: &mut Storage,
) {
    for incorrect_spelling in config
        .insensitive_corrections
        .iter()
        .chain(config.sensitive_corrections.iter())
    {
        let body = config
            .correction_text
            .replacen("{}", sender.localpart(), 1)
            .replacen("{}", &incorrect_spelling.to_string(), 1);
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
            }
            Err(e) => {
                error!("{:?}", e);
            }
        }
    }
}
