use crate::config::{Config, Storage};
use crate::helpers::{clean_text, convert_unit};
use crate::regex::UNIT_CONVERSION;

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
    config: &Config,
) -> Result<()> {
    let mut conversions = Vec::new();
    match &text.formatted_body {
        Some(v) => {
            let clean_text = clean_text(v);
            if UNIT_CONVERSION.is_match(&clean_text) {
                for cap in UNIT_CONVERSION.captures_iter(&clean_text.to_lowercase()) {
                    process_capture(&cap, &config, &mut conversions)
                }
            } else {
                debug!("There are no remaining matches after cleaning tags. Doing nothing.");
                return Ok(());
            }
        }
        None => {
            for cap in UNIT_CONVERSION.captures_iter(&text.body.to_lowercase()) {
                process_capture(&cap, &config, &mut conversions)
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
                format: None,
                formatted_body: None,
            }))
            .into_json(),
        })
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("{:?}", e);
            Ok(())
        }
    }
}

fn process_capture(capture: &regex::Captures, config: &Config, conversions: &mut Vec<(String, String)>) {
    trace!("Capture being processed is {:?}", capture);
    if !config.unit_conversion_exclusion.is_empty() {
        if capture_not_excluded(&capture, &config) {
            conversions.push((capture[1].to_string(), capture[2].to_string()))
        } else {
            trace!("Capture excluded due to exclusion rules");
        }
    } else {
        conversions.push((capture[1].to_string(), capture[2].to_string()))
    }
}

fn capture_not_excluded(capture: &regex::Captures, config: &Config) -> bool {
    for exclusion in &config.unit_conversion_exclusion {
        trace!("Exclusion this loop: {:?}", exclusion);

        if capture[0].contains(exclusion) {
            return false;
        }
    }
    true
}