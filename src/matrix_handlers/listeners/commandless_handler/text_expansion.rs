//! Performs expansion of text in messages and builds proper response

use crate::config::MatrixListenerConfig;
use crate::helpers::{clean_text, MatrixNoticeResponse};
use crate::regex::TEXT_EXPANSION;
use ruma::events::room::message::TextMessageEventContent;
use tracing::{debug, error, trace};

/// Finds and expands text plus builds response text
pub fn text_expansion(
    text: &TextMessageEventContent,
    config: &MatrixListenerConfig,
    notice_response: &mut MatrixNoticeResponse,
) {
    let mut expanded_text: Vec<String> = Vec::new();
    match &text.formatted {
        Some(v) => {
            let clean_text = clean_text(&v.body);
            if TEXT_EXPANSION.is_match(&clean_text) {
                determine_expanded_text(config, &clean_text, &mut expanded_text)
            } else {
                debug!("There are no remaining matches after cleaning tags. Doing nothing.");
                return;
            }
        }
        None => determine_expanded_text(config, &text.body, &mut expanded_text),
    }
    if expanded_text.is_empty() {
        debug!("No text to send after processing.");
    } else {
        notice_response.set_expanded_text(expanded_text);
    }
}

fn determine_expanded_text(
    config: &MatrixListenerConfig,
    text: &str,
    expanded_text: &mut Vec<String>,
) {
    for cap in TEXT_EXPANSION.captures_iter(&text.to_lowercase()) {
        trace!("{:?}", cap);
        match config.text_expansions.get(&cap[1]) {
            Some(v) => {
                expanded_text.push(v.to_string());
            }
            None => error!("Somehow lost expanded text between regex match and insertion!"),
        }
    }
}
