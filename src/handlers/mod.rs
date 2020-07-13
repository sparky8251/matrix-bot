//! Handlers for various event types
//!
//! Sub modules exist for performing various processes such as unit conversion
//! and searching github

mod commandless_handler;
mod help_handler;
mod invite_handler;
mod unit_conversion_handler;

use self::commandless_handler::commandless_handler;
use self::help_handler::help_handler;
use self::unit_conversion_handler::unit_conversion_handler;
use crate::config::{Config, Storage};

use invite_handler::{accept_invite, reject_invite};

use log::{debug, trace};
use ruma_client::{
    events::room::message::TextMessageEventContent,
    identifiers::{RoomId, UserId},
    HttpsClient,
};

/// Dispatches incoming text events to a number of different handlers depending on various conditions
pub async fn handle_text_event(
    text: &TextMessageEventContent,
    sender: &UserId,
    room_id: &RoomId,
    client: &HttpsClient,
    storage: &mut Storage,
    config: &Config,
    api_client: &reqwest::Client,
) {
    if !&text.body.starts_with('!') {
        debug!("Entering no command path...");
        commandless_handler(text, sender, room_id, client, storage, config, api_client).await
    } else if text.body.to_lowercase().starts_with("!convert ") {
        debug!("Entering unit conversion path...");
        unit_conversion_handler(text, room_id, client, storage).await
    } else if text.body.to_lowercase().starts_with("!help") {
        debug!("Entering help path...");
        help_handler(text, room_id, client, storage, config).await
    } else {
        debug!("Doing nothing...");
    }
}

/// Accepts or rejects invites to rooms from matrix users
pub async fn handle_invite_event(
    sender: &UserId,
    room_id: &RoomId,
    client: &HttpsClient,
    config: &Config,
) {
    trace!("Invited by {} to room {} ", &sender, &room_id);
    if config.admins.contains(&sender) {
        accept_invite(&sender, &room_id, &client).await;
    } else {
        reject_invite(&sender, &room_id, &client).await;
    }
}
