mod correct_spelling;
mod helpers;

use self::correct_spelling::correct_spelling_check;
use self::helpers::do_nothing;
use crate::session::SavedSession;

use anyhow::Result;
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
        correct_spelling_check(text, sender, room_id, client, session).await
    } else {
        do_nothing().await
    }
}
