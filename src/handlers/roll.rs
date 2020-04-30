use super::helpers::do_nothing;
use crate::session::SavedSession;

use anyhow::Result;
use log::error;
use nom::{named, tag_no_case, take_until};
use rand::Rng;
use ruma_client::{
    api::r0::message::create_message_event,
    events::{
        room::message::{MessageEventContent, TextMessageEventContent},
        EventJson, EventType,
    },
    identifiers::RoomId,
    HttpsClient,
};

#[allow(dead_code)]
pub(super) async fn roll(
    text: &TextMessageEventContent,
    room_id: &RoomId,
    client: &HttpsClient,
    session: &mut SavedSession,
) -> Result<()> {
    if text.relates_to == None && text.formatted_body == None {
        named!(strip_tag, tag_no_case!("!roll "));
        let message = match strip_tag(&text.body.as_bytes()) {
            Ok(v) => v.0,
            Err(e) => {
                error!("{:?}", e);
                return Ok(())
            }
        };

        // FIXME: Current fails to parse `!roll 20` correctly at all
        // and parses `!roll 20 40` as a `!roll 20`.
        // Also needs to have roll bounds set so `!roll 40 20` isnt accepted
        named!(split_parts, take_until!("-"));
        let (start, end) = match split_parts(message) {
            Ok(v) => (Some(v.1.to_vec()), Some(v.0.to_vec())),
            Err(e) => {
                error!("{:?}", e);
                (None, None)
            }
        };

        let start = match start {
            Some(s) => match String::from_utf8(s) {
                Ok(v) => match v.split('-').collect::<Vec<&str>>()[0].trim().parse::<i64>() {
                    Ok(n) => Some(n),
                    Err(e) => {
                        error!("{:?}", e);
                        None
                    }
                },
                Err(e) => {
                    error!("{:?}", e);
                    None
                }
            },
            None => {
                error!("No quantity provided!");
                None
            }
        };

        let end = match end {
            Some(s) => match String::from_utf8(s) {
                Ok(v) => match v.split('-').collect::<Vec<&str>>()[0].trim().parse::<i64>() {
                    Ok(n) => Some(n),
                    Err(e) => {
                        error!("{:?}", e);
                        None
                    }
                },
                Err(e) => {
                    error!("{:?}", e);
                    None
                }
            },
            None => {
                error!("No quantity provided!");
                None
            }
        };

        match end {
            Some(en) => match start {
                Some(st) => {
                    let roll = rand::thread_rng().gen_range(st, en);
                    return send_roll(roll, room_id, client, session).await;
                }
                None => {
                    error!("No start and end value provided!");
                    return do_nothing().await;
                }
            },
            None => match start {
                Some(en) => {
                    let roll = rand::thread_rng().gen_range(1, en);
                    return send_roll(roll, room_id, client, session).await;
                }
                None => {
                    error!("No end value provided!");
                    return do_nothing().await;
                }
            },
        }
    } else {
        do_nothing().await
    }
}

#[allow(dead_code)]
async fn send_roll(
    roll: i64,
    room_id: &RoomId,
    client: &HttpsClient,
    session: &mut SavedSession,
) -> Result<()> {
    let response = client
        .request(create_message_event::Request {
            room_id: room_id.clone(), // INVESTIGATE: Does this really need to be cloned?
            event_type: EventType::RoomMessage,
            txn_id: session.next_txn_id(),
            data: EventJson::from(MessageEventContent::Text(TextMessageEventContent {
                body: format!("{}", roll),
                format: None,
                formatted_body: None,
                relates_to: None,
            })),
        })
        .await;
    match response {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("{:?}", e);
            Ok(())
        }
    }
}
