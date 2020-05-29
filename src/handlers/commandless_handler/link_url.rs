use crate::config::{Config, Storage};
use crate::helpers::clean_text;
use crate::regex::LINK_URL;

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

pub async fn link_url(
    text: &TextMessageEventContent,
    room_id: &RoomId,
    client: &HttpsClient,
    storage: &mut Storage,
    config: &Config,
) -> Result<()> {
    let mut links: Vec<String> = Vec::new();
    match &text.formatted_body {
        Some(v) => {
            let clean_text = clean_text(v);
            if LINK_URL.is_match(&clean_text) {
                for cap in LINK_URL.captures_iter(&clean_text.to_lowercase()) {
                    trace!("{:?}", cap);
                    if config.linkers.contains(&cap[1].to_lowercase()) {
                        match config.links.get(&cap[2].to_string()) {
                            Some(v) => {
                                links.push(v.to_string())
                            }
                            None => error!("Somehow lost link between matching it and inserting it into reply list!")
                        }
                    } else {
                        debug!("No link found for {}", cap[2].to_string())
                    }
                }
            } else {
                debug!("There are no remaining matches after cleaning tags. Doing nothing.");
                return Ok(());
            }
        }
        None => {
            for cap in LINK_URL.captures_iter(&text.body.to_lowercase()) {
                trace!("{:?}", cap);
                if config.linkers.contains(&cap[1].to_lowercase()) {
                    match config.links.get(&cap[2].to_string()) {
                        Some(v) => {
                            links.push(v.to_string())
                        }
                        None => error!("Somehow lost link between matching it and inserting it into reply list!")
                    }
                } else {
                    debug!("No link found for {}", cap[2].to_string())
                }
            }
        }
    }
    let mut results = String::new();
    if links.is_empty() {
        debug!("No links to build response with after processing");
        return Ok(());
    }
    for result in links {
        results.push_str(&result);
        results.push('\n');
    }
    let results = results;
    match client
        .request(create_message_event::Request {
            room_id: room_id.clone(), //FIXME: There has to be a better way than to clone here
            event_type: EventType::RoomMessage,
            txn_id: storage.next_txn_id(),
            data: EventJson::from(MessageEventContent::Notice(NoticeMessageEventContent {
                body: results,
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
