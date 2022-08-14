use crate::config::ResponderStorage;
use crate::matrix::MatrixClient;
use ruma::{
    api::client::{
        membership::{join_room_by_id, leave_room},
        message::send_message_event,
    },
    events::room::message::RoomMessageEventContent,
    RoomId, UserId,
};
use tracing::{debug, error, info};

pub async fn send_notice(
    client: &MatrixClient,
    room_id: &RoomId,
    storage: &mut ResponderStorage,
    message: String,
) {
    let content = RoomMessageEventContent::notice_plain(message);
    let next_txn_id = storage.next_txn_id();
    let req = send_message_event::v3::Request::new(room_id, &next_txn_id, &content)
        .expect("m.room.message serialization must work");
    if let Err(e) = client.send_request(req).await {
        error!("{:?}", e);
    }
}
pub async fn send_plain_text(
    room_id: &RoomId,
    storage: &mut ResponderStorage,
    message: String,
    client: &MatrixClient,
) {
    let content = RoomMessageEventContent::text_plain(message);
    let next_txn_id = storage.next_txn_id();
    let req = send_message_event::v3::Request::new(room_id, &next_txn_id, &content)
        .expect("m.room.message serialization must work");
    if let Err(e) = client.send_request(req).await {
        error!("Unable to send response due to error {:?}", e);
    }
}

pub async fn send_formatted_text(
    room_id: &RoomId,
    storage: &mut ResponderStorage,
    message: String,
    formatted_message: Option<String>,
    client: &MatrixClient,
) {
    let content =
        RoomMessageEventContent::text_html(message, formatted_message.unwrap_or_default());
    let next_txn_id = storage.next_txn_id();
    let req = send_message_event::v3::Request::new(room_id, &next_txn_id, &content)
        .expect("m.room.message serialization must work");
    if let Err(e) = client.send_request(req).await {
        error!("Unable to send response due to error {:?}", e);
    }
}

pub async fn send_formatted_notice(
    room_id: &RoomId,
    storage: &mut ResponderStorage,
    message: String,
    formatted_message: Option<String>,
    client: &MatrixClient,
) {
    let content =
        RoomMessageEventContent::notice_html(message, formatted_message.unwrap_or_default());
    let next_txn_id = storage.next_txn_id();
    let req = send_message_event::v3::Request::new(room_id, &next_txn_id, &content)
        .expect("m.room.message serialization must work");
    if let Err(e) = client.send_request(req).await {
        error!("{:?}", e);
    }
}

pub async fn accept_invite(sender: &UserId, room_id: &RoomId, client: &MatrixClient) {
    info!("Authorized user {} invited me to room {}", sender, room_id);
    let response = client
        .send_request(join_room_by_id::v3::Request::new(room_id))
        .await;
    match response {
        Ok(_) => info!("Successfully joined room {}", room_id),
        Err(e) => debug!("Unable to join room {} because of error {:?}", room_id, e),
    }
}

/// Will reject an invite and print the user that tried to logs
pub async fn reject_invite(sender: &UserId, room_id: &RoomId, client: &MatrixClient) {
    let response = client
        .send_request(leave_room::v3::Request::new(room_id))
        .await;
    match response {
        Ok(_) => info!("Rejected invite from unathorized user {}", sender),
        Err(e) => debug!("Unable to reject invite this loop because of error {:?}", e),
    }
}
