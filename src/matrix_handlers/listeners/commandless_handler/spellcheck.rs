//! Performs spellcheck and build an appropriate correction text

use crate::config::{MatrixListenerConfig, SpellCheckKind};

use ruma_client::{events::room::message::TextMessageEventContent, identifiers::UserId};

/// Checks that provided string contains an incorrect spelling and returns a formatted string that corrects the user
pub fn spellcheck(
    text: &TextMessageEventContent,
    sender: &UserId,
    config: &MatrixListenerConfig,
) -> Option<String> {
    let mut result = String::new();
    for i in config.incorrect_spellings.iter() {
        match i {
            SpellCheckKind::SpellCheckInsensitive(v) => {
                if text.body.contains(&v.to_string().to_lowercase()) {
                    result = config
                        .correction_text
                        .replacen("{}", sender.localpart(), 1)
                        .replacen("{}", &v.to_string(), 1);
                }
            }
            SpellCheckKind::SpellCheckSensitive(v) => {
                if text.body.contains(&v.to_string()) {
                    result = config
                        .correction_text
                        .replacen("{}", sender.localpart(), 1)
                        .replacen("{}", &v.to_string(), 1);
                }
            }
        }
    }
    let result = result;
    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}
