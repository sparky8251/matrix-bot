mod commandless_handler;
mod invite_handler;
mod unit_conversion_handler;

use self::commandless_handler::commandless_handler;
use self::unit_conversion_handler::unit_conversion_handler;
use crate::regex::{NO_BANG, UNIT_CONVERSION_COMMAND};
use crate::config::{Config, Storage};

use invite_handler::{accept_invite, reject_invite};

use anyhow::Result;
use log::{debug, trace};
use ruma_client::{
    events::room::message::TextMessageEventContent,
    identifiers::{RoomId, UserId},
    HttpsClient,
};

pub async fn handle_text_event(
    text: &TextMessageEventContent,
    sender: &UserId,
    room_id: &RoomId,
    client: &HttpsClient,
    storage: &mut Storage,
    config: &Config,
    api_client: &reqwest::Client,
) -> Result<()> {
    if NO_BANG.is_match(&text.body) {
        debug!("Entering no command path...");
        commandless_handler(text, sender, room_id, client, storage, config, api_client).await
    } else if UNIT_CONVERSION_COMMAND.is_match(&text.body.to_lowercase()) {
        debug!("Entering unit conversion path...");
        unit_conversion_handler(text, room_id, client, storage).await
    } else {
        debug!("Entering do nothing path...");
        Ok(())
    }
}

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
