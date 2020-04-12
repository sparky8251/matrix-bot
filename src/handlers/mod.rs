mod correct_spelling;
mod helpers;
mod roll;
mod unit_conversion;

use self::correct_spelling::correct_spelling_check;
use self::helpers::do_nothing;
#[allow(unused_imports)]
use self::roll::roll;
use self::unit_conversion::unit_conversion;
use crate::session::SavedSession;

use anyhow::Result;
use log::debug;
use ruma_client::HttpsClient;
use ruma_events::room::message::TextMessageEventContent;
use ruma_identifiers::{RoomId, UserId};

pub async fn handle_text_message(
    text: &TextMessageEventContent,
    sender: &UserId,
    room_id: &RoomId,
    client: &HttpsClient,
    session: &mut SavedSession,
) -> Result<()> {
    if !text.body.starts_with('!') {
        debug!("Entering spell check path...");
        correct_spelling_check(text, sender, room_id, client, session).await
    } else if text.body.to_lowercase().starts_with("!convert ") {
        debug!("Entering unit conversion path...");
        unit_conversion(text, room_id, client, session).await
    } else if text.body.to_lowercase().starts_with("!roll ") {
        debug!("Entering roll path...");
        // roll(text, room_id, client, session).await
        do_nothing().await
    } else {
        debug!("Entering do nothing path...");
        do_nothing().await
    }
}
