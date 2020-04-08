use crate::error::Result;
use crate::session::SavedSession;

use std::fmt::{Display, Formatter};
use std::process;
use std::time::SystemTime;

use lazy_static::lazy_static;
use log::error;
use ruma_client::{api::r0::message::create_message_event, HttpsClient};
use ruma_events::{
    room::message::{MessageEventContent, TextMessageEventContent},
    EventType,
};
use ruma_identifiers::{RoomId, UserId};

#[derive(Clone)]
enum SpellCheckKind {
    SpellCheckInsensitive(InsensitiveSpelling),
    SpellCheckSensitive(SensitiveSpelling),
}

#[derive(Clone)]
struct SpellCheck {
    insensitive: Vec<InsensitiveSpelling>,
    sensitive: Vec<SensitiveSpelling>,
}
#[derive(Clone)]
struct InsensitiveSpelling {
    spelling: String,
}

#[derive(Clone)]
struct SensitiveSpelling {
    spelling: String,
}

impl SpellCheck {
    fn new() -> Self {
        SpellCheck {
            insensitive: vec![
                InsensitiveSpelling::from("Jellyfish"),
                InsensitiveSpelling::from("Jelly Fin"),
            ],
            sensitive: vec![
                SensitiveSpelling::from("JellyFin"),
                SensitiveSpelling::from("jellyFin"),
            ],
        }
    }
}

impl From<&str> for InsensitiveSpelling {
    fn from(str: &str) -> Self {
        InsensitiveSpelling {
            spelling: str.to_string(),
        }
    }
}

impl From<&str> for SensitiveSpelling {
    fn from(str: &str) -> Self {
        SensitiveSpelling {
            spelling: str.to_string(),
        }
    }
}

impl Display for InsensitiveSpelling {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.spelling)
    }
}

impl Display for SensitiveSpelling {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.spelling)
    }
}

impl Display for SpellCheckKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SpellCheckKind::SpellCheckInsensitive(v) => write!(f, "{}", v),
            SpellCheckKind::SpellCheckSensitive(v) => write!(f, "{}", v),
        }
    }
}

lazy_static! {
    static ref SPELL_CHECK: SpellCheck = SpellCheck::new();
}

pub(super) async fn correct_spelling_check(
    text: &TextMessageEventContent,
    sender: &UserId,
    room_id: &RoomId,
    client: &HttpsClient,
    session: &mut SavedSession,
) -> Result<()> {
    if sender.localpart() == session.get_username() {
        // do nothing
    } else {
        if session.correction_time_cooldown() && text.relates_to == None {
            for incorrect_spelling in SPELL_CHECK
                .insensitive
                .clone()
                .into_iter()
                .map(SpellCheckKind::SpellCheckInsensitive)
                .chain(
                    SPELL_CHECK
                        .sensitive
                        .clone()
                        .into_iter()
                        .map(SpellCheckKind::SpellCheckSensitive),
                )
            {
                let incorrect = match incorrect_spelling.clone() {
                    SpellCheckKind::SpellCheckInsensitive(v) => text
                        .body
                        .to_lowercase()
                        .contains(&v.to_string().to_lowercase()),
                    SpellCheckKind::SpellCheckSensitive(v) => text.body.contains(&v.to_string()),
                };
                if incorrect {
                    let response = client
                        .request(create_message_event::Request {
                            room_id: room_id.clone(),
                            event_type: EventType::RoomMessage,
                            txn_id: session.next_txn_id(),
                            data: MessageEventContent::Text(TextMessageEventContent {
                                body: correct_spelling(
                                    sender.localpart(),
                                    &incorrect_spelling.to_string(),
                                ),
                                format: None,
                                formatted_body: None,
                                relates_to: None,
                            }),
                        })
                        .await;
                    match response {
                        Ok(_) => {
                            session.set_last_correction_time(SystemTime::now());
                            return Ok(());
                        }
                        Err(e) => {
                            error!("{:?}", e);
                            process::exit(48)
                        }
                    }
                }
            }
        }
    }
    return Ok(()); // No matches found or cooldown time not met, so return Ok
}

fn correct_spelling(user: &str, incorrect_spelling: &str) -> String {
    format!("I'd just like to interject for a moment {}. What you're referring to as {}, is in fact, Jellyfin, or as I've recently taken to calling it, Linux plus Jellyfin. Jellyfin is not an operating system unto itself, but rather another free component of a fully functioning Linux system made useful by ffmpeg, dwarf fortress, and other vital system components comprising a full OS as defined by Steve Ballmer. Many computer users run a modified version of Jellyfin every day, without realizing it. Through a peculiar turn of events, the version of Jellyfin which is widely used today is often called \"that one media player thingy you have\", and many of its users are not aware that it is basically the Jellyfin system, developed with slave labor. There really is a Jellyfin, and these people are using it, but it is just a part of the system they use. Jellyfin is the answer: the program in the system that allocates the machine's resources until your fans are screaming for the sweet release of death. The media server is an essential part of an operating system, but useless by itself; it can only function in the context of a complete collection of Linux ISOs. Jellyfin is normally used in combination with the Linux operating system: the whole system is basically Linux with Jellyfin added, or Linux/Jellyfin. All the so-called \"Linux\" releases are really distributions of Linux/Jellyfin.", user, incorrect_spelling)
}
