use crate::config::ResponderStorage;
use ruma::{
    api::client::r0::{
        membership::{join_room_by_id, leave_room},
        message::send_message_event,
    },
    events::{
        room::message::{
            FormattedBody, MessageEventContent, NoticeMessageEventContent, TextMessageEventContent,
        },
        EventType,
    },
    Raw, RoomId, UserId,
};
use ruma_client::HttpsClient;
use tracing::{debug, error, info};

pub async fn send_notice(
    client: &HttpsClient,
    room_id: &RoomId,
    storage: &mut ResponderStorage,
    message: String,
) {
    match client
        .request(send_message_event::Request {
            room_id,
            event_type: EventType::RoomMessage,
            txn_id: &storage.next_txn_id(),
            data: Raw::from(MessageEventContent::Notice(
                NoticeMessageEventContent::plain(message),
            ))
            .into_json(),
        })
        .await
    {
        Ok(_) => (),
        Err(e) => error!("{:?}", e),
    }
}
pub async fn send_plain_text(
    room_id: &RoomId,
    storage: &mut ResponderStorage,
    message: String,
    client: &HttpsClient,
) {
    match client
        .request(send_message_event::Request {
            room_id: &room_id,
            event_type: EventType::RoomMessage,
            txn_id: &storage.next_txn_id(),
            data: Raw::from(MessageEventContent::Text(TextMessageEventContent::plain(
                message,
            )))
            .into_json(),
        })
        .await
    {
        Ok(_) => (),
        Err(e) => error!("Unable to send response due to error {:?}", e),
    }
}

pub async fn send_formatted_text(
    room_id: &RoomId,
    storage: &mut ResponderStorage,
    message: String,
    formatted_message: Option<String>,
    client: &HttpsClient,
) {
    match client
        .request(send_message_event::Request {
            room_id,
            event_type: EventType::RoomMessage,
            txn_id: &storage.next_txn_id(),
            data: Raw::from(MessageEventContent::Text(TextMessageEventContent {
                body: message,
                formatted: formatted_message.map(FormattedBody::html),
                relates_to: None,
            }))
            .into_json(),
        })
        .await
    {
        Ok(_) => (),
        Err(e) => error!("Unable to send response due to error {:?}", e),
    }
}

pub async fn send_formatted_notice(
    room_id: &RoomId,
    storage: &mut ResponderStorage,
    message: String,
    formatted_message: Option<String>,
    client: &HttpsClient,
) {
    match client
        .request(send_message_event::Request {
            room_id,
            event_type: EventType::RoomMessage,
            txn_id: &storage.next_txn_id(),
            data: Raw::from(MessageEventContent::Notice(NoticeMessageEventContent {
                body: message,
                relates_to: None,
                formatted: formatted_message.map(FormattedBody::html),
            }))
            .into_json(),
        })
        .await
    {
        Ok(_) => (),
        Err(e) => error!("{:?}", e),
    }
}

pub async fn accept_invite(sender: &UserId, room_id: &RoomId, client: &HttpsClient) {
    info!("Authorized user {} invited me to room {}", sender, room_id);
    let response = client.request(join_room_by_id::Request::new(room_id)).await;
    match response {
        Ok(_) => info!("Successfully joined room {}", room_id),
        Err(e) => debug!("Unable to join room {} because of error {:?}", room_id, e),
    }
}

/// Will reject an invite and print the user that tried to logs
pub async fn reject_invite(sender: &UserId, room_id: &RoomId, client: &HttpsClient) {
    let response = client.request(leave_room::Request::new(room_id)).await;
    match response {
        Ok(_) => info!("Rejected invite from unathorized user {}", sender),
        Err(e) => debug!("Unable to reject invite this loop because of error {:?}", e),
    }
}
