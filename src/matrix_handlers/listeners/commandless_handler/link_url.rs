//! Performs lookup of URLs in message text and builds proper response

use crate::config::MatrixListenerConfig;
use crate::helpers::{clean_text, MatrixNoticeResponse};
use crate::regex::LINK_URL;

use ruma_client::events::room::message::TextMessageEventContent;
use tracing::{debug, error, trace};
use url::Url;

/// Finds and links URLs requested and builds response text
pub fn link_url(
    text: &TextMessageEventContent,
    config: &MatrixListenerConfig,
    notice_response: &mut MatrixNoticeResponse,
) {
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
                return;
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

    if links.is_empty() {
        debug!("No links to build response with after processing");
    } else {
        let mut results = Vec::new();
        for result in links {
            match Url::parse(&result) {
                Ok(v) => results.push(v),
                Err(e) => error!(
                    "Unable to parse result {:?} to Url due to error {:?}",
                    result, e
                ),
            }
        }
        notice_response.set_links(results);
    }
}
