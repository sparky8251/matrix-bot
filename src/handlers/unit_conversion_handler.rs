use crate::config::Storage;
use crate::helpers::convert_unit;
use crate::regex::UNIT_CONVERSION;

use anyhow::Result;
use log::{debug, error};
use ruma_client::{
    api::r0::message::create_message_event,
    events::{
        room::message::{MessageEventContent, NoticeMessageEventContent, TextMessageEventContent},
        EventJson, EventType,
    },
    identifiers::RoomId,
    HttpsClient,
};

pub(super) async fn unit_conversion_handler(
    text: &TextMessageEventContent,
    room_id: &RoomId,
    client: &HttpsClient,
    storage: &mut Storage,
) -> Result<(), anyhow::Error> {
    if text.relates_to == None && text.formatted_body == None {
        let mut conversions = Vec::new();
        for cap in UNIT_CONVERSION.captures_iter(&text.body.to_lowercase()) {
            conversions.push((cap[1].to_string(), cap[2].to_string()));
        }
        let result = match convert_unit(conversions) {
            Some(v) => {
                let mut result = String::new();
                for converted in v {
                    result.push_str(&converted.to_string());
                    result.push_str("\n");
                }
                result.trim().to_string()
            }
            None => {
                debug!("No convertable units found. No reply will be constructed.");
                return Ok(());
            }
        };

        match client
            .request(create_message_event::Request {
                room_id: room_id.clone(), //FIXME: There has to be a better way than to clone here
                event_type: EventType::RoomMessage,
                txn_id: storage.next_txn_id(),
                data: EventJson::from(MessageEventContent::Notice(NoticeMessageEventContent {
                    body: result,
                    relates_to: None,
                })),
            })
            .await
        {
            Ok(_) => return Ok(()),
            Err(e) => {
                error!("{:?}", e);
                return Ok(());
            }
        }
    }
    Ok(())
}
