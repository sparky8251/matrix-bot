use crate::macros::convert_unit;
use crate::regex::{CODE_TAG, PRE_TAG, UNIT_CONVERSION};
use crate::Storage;

use anyhow::Result;
use log::{debug, error, trace};
use ruma_client::{
    api::r0::message::create_message_event,
    events::{
        room::message::{MessageEventContent, NoticeMessageEventContent, TextMessageEventContent},
        EventJson, EventType,
    },
    identifiers::RoomId,
    HttpsClient,
};

pub async fn unit_conversion(
    text: &TextMessageEventContent,
    room_id: &RoomId,
    client: &HttpsClient,
    storage: &mut Storage,
) -> Result<()> {
    let mut conversions = Vec::new();
    match &text.formatted_body {
        Some(v) => {
            let clean_text = CODE_TAG.replace_all(&v, "");
            trace!("Cleaned text after code tag is {:?}", clean_text);
            let clean_text = PRE_TAG.replace_all(&clean_text, "");
            trace!("Cleaned text after pre tag is {:?}", clean_text);
            if UNIT_CONVERSION.is_match(&clean_text) {
                for cap in UNIT_CONVERSION.captures_iter(&clean_text.to_lowercase()) {
                    trace!("{:?}", cap);
                    conversions.push((cap[1].to_string(), cap[2].to_string()))
                }
            } else {
                debug!("There are no remaining matches after cleaning tags. Doing nothing.");
                return Ok(());
            }
        }
        None => {
            for cap in UNIT_CONVERSION.captures_iter(&text.body.to_lowercase()) {
                conversions.push((cap[1].to_string(), cap[2].to_string()))
            }
        }
    }
    let conversions = conversions;
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
