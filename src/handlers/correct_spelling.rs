use std::fmt::{Display, Formatter};
use std::time::SystemTime;

use crate::session::SavedSession;

use anyhow::Result;
use lazy_static::lazy_static;
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
                .clone() //FIXME: There has to be a better way than to clone here
                .into_iter()
                .map(SpellCheckKind::SpellCheckInsensitive)
                .chain(
                    SPELL_CHECK
                        .sensitive
                        .clone() //FIXME: There has to be a better way than to clone here
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
                            room_id: room_id.clone(), // INVESTIGATE: Does this really need to be cloned?
                            event_type: EventType::RoomMessage,
                            txn_id: session.next_txn_id(),
                            data: EventJson::from(MessageEventContent::Text(
                                TextMessageEventContent {
                                    body: correct_spelling(
                                        sender.localpart(),
                                        &incorrect_spelling.to_string(),
                                    ),
                                    format: None,
                                    formatted_body: None,
                                    relates_to: None,
                                },
                            )),
                        })
                        .await;
                    match response {
                        Ok(_) => {
                            session.set_last_correction_time(SystemTime::now());
                            return Ok(());
                        }
                        Err(e) => {
                            error!("{:?}", e);
                            return Ok(());
                        }
                    }
                }
            }
        }
    }
    return Ok(()); // No matches found or cooldown time not met, so return Ok
}

fn correct_spelling(user: &str, incorrect_spelling: &str) -> String {
    format!("I'd just like to interject for a moment {}. What you're referring to as {}, is in fact, Jellyfin, or as I've recently taken to calling it, Emby plus Jellyfin. Jellyfin is not a media server unto itself, but a free component of a media server as defined by Luke Pulverenti. Through a peculiar turn of events, the version of Jellyfin which is widely used today is basically developed with slave labor. Please recognize the harm caused to the slaves by misnaming the project.", user, incorrect_spelling)
}
