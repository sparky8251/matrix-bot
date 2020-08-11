//! Performs unit conversions and adds them to response data

use crate::config::MatrixListenerConfig;
use crate::helpers::{clean_text, convert_unit, MatrixNoticeResponse};
use crate::regex::UNIT_CONVERSION;
use ruma_client::events::room::message::TextMessageEventContent;
use tracing::{debug, trace};

/// Adds unit conversions to the supplied BotResponseNotice
pub fn unit_conversion(
    text: &TextMessageEventContent,
    config: &MatrixListenerConfig,
    notice_response: &mut MatrixNoticeResponse,
) {
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
                return;
            }
        }
        None => {
            for cap in UNIT_CONVERSION.captures_iter(&text.body.to_lowercase()) {
                process_capture(&cap, &config, &mut conversions)
            }
        }
    }
    let conversions = conversions;
    match convert_unit(conversions) {
        Some(v) => notice_response.set_unit_conversions(v),
        None => debug!("No convertable units found. No unit conversions will be performed."),
    }
}

/// Processes a unit conversion regex capture into a Vec
fn process_capture(
    capture: &regex::Captures,
    config: &MatrixListenerConfig,
    conversions: &mut Vec<(String, String)>,
) {
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

/// Verifies if a capture will be excluded from conversion because of a space between the quantity and unit
fn capture_not_excluded(capture: &regex::Captures, config: &MatrixListenerConfig) -> bool {
    for exclusion in &config.unit_conversion_exclusion {
        trace!("Exclusion this loop: {:?}", exclusion);
        if capture[0].contains(exclusion) {
            return false;
        }
    }
    true
}
