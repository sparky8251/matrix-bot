use ruma_client::{api::r0::message::create_message_event, HttpsClient};
use ruma_events::{
    room::message::{MessageEventContent, TextMessageEventContent},
    EventType,
};
use ruma_identifiers::{RoomId, UserId};

use crate::error::{Error, Result};
use crate::session::SavedSession;

pub async fn handle_text_message(
    text: &TextMessageEventContent,
    sender: &UserId,
    room_id: &RoomId,
    client: &HttpsClient,
    session: &mut SavedSession,
) -> Result<()> {
    if !text.body.starts_with('!') {
        correct_spelling(text, sender, room_id, client, session).await
    } else {
        do_nothing().await
    }
}

async fn correct_spelling(
    text: &TextMessageEventContent,
    sender: &UserId,
    room_id: &RoomId,
    client: &HttpsClient,
    session: &mut SavedSession,
) -> Result<()> {
    if text.body.contains("jellyfish") {
        if sender.localpart() == session.get_username() {
            do_nothing().await
        } else {
            if text.relates_to == None {
                let response = client
                    .request(create_message_event::Request {
                        room_id: room_id.clone(),
                        event_type: EventType::RoomMessage,
                        txn_id: session.next_txn_id(),
                        data: MessageEventContent::Text(TextMessageEventContent {
                            body: text.body.to_string(),
                            format: None,
                            formatted_body: None,
                            relates_to: None,
                        }),
                    })
                    .await;
                match response {
                    Ok(_) => Ok(()),
                    Err(e) => Err(Error::RumaClientError(e)),
                }
            } else {
                do_nothing().await
            }
        }
    } else {
        do_nothing().await
    }
}

async fn do_nothing() -> Result<()> {
    Ok(())
}
