//! Performs group pings based on message text and builds proper response

use crate::config::MatrixListenerConfig;
use crate::helpers::{clean_text, MatrixFormattedTextResponse};
use crate::regex::GROUP_PING;
use ruma::{events::room::message::TextMessageEventContent, UserId};
use std::collections::HashSet;
use tracing::{debug, error, trace};

/// Finds requested users to ping and builds response text
pub fn group_ping(
    text: &TextMessageEventContent,
    sender: &UserId,
    config: &MatrixListenerConfig,
    text_response: &mut MatrixFormattedTextResponse,
) {
    let mut users: HashSet<UserId> = HashSet::new();
    if !config.group_ping_users.contains(sender) {
        debug!("User not authorized for group pings. Ignoring...");
        return;
    }
    match &text.formatted {
        Some(v) => {
            let clean_text = clean_text(&v.body);
            if GROUP_PING.is_match(&clean_text) {
                determine_users(&config, &clean_text, &mut users)
            } else {
                debug!("There are no remaining matches after cleaning tags. Doing nothing.");
                return;
            }
        }
        None => determine_users(&config, &text.body, &mut users),
    }
    if users.is_empty() {
        debug!("No users to ping after processing.");
    } else {
        // Remove user that requested ping if they exist in the list AND arent the only one in the list
        if users.len() != 1 {
            users.remove(&sender);
        }
        text_response.set_users(users);
    }
}

fn determine_users(config: &MatrixListenerConfig, text: &str, users: &mut HashSet<UserId>) {
    for cap in GROUP_PING.captures_iter(&text.to_lowercase()) {
        trace!("{:?}", cap);
        if cap[1].eq("%all") {
            for user in config.group_pings.values().flatten() {
                users.insert(user.clone());
            }
        } else {
            match config.group_pings.get(&cap[1]) {
                Some(v) => {
                    for user in v {
                        users.insert(user.clone());
                    }
                }
                None => error!("Somehow lost group between regex match and insertion!"),
            }
        }
    }
}
