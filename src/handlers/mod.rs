mod helpers;
mod no_command;
mod unit_conversion;

use self::helpers::do_nothing;
use self::no_command::no_command_check;
use self::unit_conversion::unit_conversion;
use crate::regex::{NO_BANG, SINGLE_UNIT_CONVERSION};
use crate::{BotConfig, Storage};

use anyhow::Result;
use log::debug;
use ruma_client::{
    events::room::message::TextMessageEventContent,
    identifiers::{RoomId, UserId},
    HttpsClient,
};

pub async fn handle_text_message(
    text: &TextMessageEventContent,
    sender: &UserId,
    room_id: &RoomId,
    client: &HttpsClient,
    storage: &mut Storage,
    config: &BotConfig,
    api_client: &reqwest::Client,
) -> Result<()> {
    if NO_BANG.is_match(&text.body) {
        debug!("Entering no command path...");
        no_command_check(text, sender, room_id, client, storage, config, api_client).await
    } else if SINGLE_UNIT_CONVERSION.is_match(&text.body.to_lowercase()) {
        debug!("Entering unit conversion path...");
        unit_conversion(text, room_id, client, storage).await
    } else {
        debug!("Entering do nothing path...");
        do_nothing().await
    }
}
