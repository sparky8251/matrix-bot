use super::helpers::do_nothing;

use crate::error::{Error, Result};
use crate::session::SavedSession;

use ruma_client::{api::r0::message::create_message_event, HttpsClient};
use ruma_events::{
    room::message::{MessageEventContent, TextMessageEventContent},
    EventType,
};
use ruma_identifiers::{RoomId, UserId};

static INSENSITIVE_SPELL_CHECK: &'static [&'static str] = &["Jellyfish", "Jelly Fin"];
static SENSITIVE_SPELL_CHECK: &'static [&'static str] = &["JellyFin", "jellyFin"];

pub(super) async fn correct_spelling_check(
    text: &TextMessageEventContent,
    sender: &UserId,
    room_id: &RoomId,
    client: &HttpsClient,
    session: &mut SavedSession,
) -> Result<()> {
    if sender.localpart() == session.get_username() {
        do_nothing().await
    } else {
        if text.relates_to == None {
            for incorrect_spelling in INSENSITIVE_SPELL_CHECK.into_iter() {
                if text
                    .body
                    .to_lowercase()
                    .contains(&incorrect_spelling.to_lowercase())
                {
                    let response = client
                        .request(create_message_event::Request {
                            room_id: room_id.clone(),
                            event_type: EventType::RoomMessage,
                            txn_id: session.next_txn_id(),
                            data: MessageEventContent::Text(TextMessageEventContent {
                                body: correct_spelling(sender.localpart(), incorrect_spelling),
                                format: None,
                                formatted_body: None,
                                relates_to: None,
                            }),
                        })
                        .await;
                    match response {
                        Ok(_) => return Ok(()),
                        Err(e) => return Err(Error::RumaClientError(e)),
                    }
                }
            }
            for incorrect_spelling in SENSITIVE_SPELL_CHECK.into_iter() {
                if text.body.contains(incorrect_spelling) {
                    let response = client
                        .request(create_message_event::Request {
                            room_id: room_id.clone(),
                            event_type: EventType::RoomMessage,
                            txn_id: session.next_txn_id(),
                            data: MessageEventContent::Text(TextMessageEventContent {
                                body: correct_spelling(sender.localpart(), incorrect_spelling),
                                format: None,
                                formatted_body: None,
                                relates_to: None,
                            }),
                        })
                        .await;
                    match response {
                        Ok(_) => return Ok(()),
                        Err(e) => return Err(Error::RumaClientError(e)),
                    }
                }
            }
        }
        return Ok(()); // No matches found, so return Ok
    }
}

fn correct_spelling(user: &str, incorrect_spelling: &str) -> String {
    format!("I'd just like to interject for a moment {}. What you're referring to as {}, is in fact, Jellyfin, or as I've recently taken to calling it, Linux plus Jellyfin. Jellyfin is not an operating system unto itself, but rather another free component of a fully functioning Linux system made useful by ffmpeg, dwarf fortress, and other vital system components comprising a full OS as defined by Steve Ballmer. Many computer users run a modified version of Jellyfin every day, without realizing it. Through a peculiar turn of events, the version of Jellyfin which is widely used today is often called \"that one media player thingy you have\", and many of its users are not aware that it is basically the Jellyfin system, developed with slave labor. There really is a Jellyfin, and these people are using it, but it is just a part of the system they use. Jellyfin is the answer: the program in the system that allocates the machine's resources until your fans are screaming for the sweet release of death. The media server is an essential part of an operating system, but useless by itself; it can only function in the context of a complete collection of Linux ISOs. Jellyfin is normally used in combination with the Linux operating system: the whole system is basically Linux with Jellyfin added, or Linux/Jellyfin. All the so-called \"Linux\" releases are really distributions of Linux/Jellyfin.", user, incorrect_spelling)
}
