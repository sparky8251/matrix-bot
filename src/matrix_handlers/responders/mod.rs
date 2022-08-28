use crate::config::ResponderStorage;
use crate::matrix::MatrixClient;
use ruma::{
    api::client::{
        membership::{ban_user, join_room_by_id, leave_room},
        message::send_message_event,
    },
    events::room::message::RoomMessageEventContent,
    OwnedRoomId, UserId,
};
use std::collections::HashSet;
use tracing::{debug, error, info};

pub async fn send_notice(
    client: &MatrixClient,
    room_id: Option<OwnedRoomId>,
    storage: &mut ResponderStorage,
    message: String,
) {
    let content = RoomMessageEventContent::notice_plain(message);
    let next_txn_id = storage.next_txn_id();
    let room_id = match room_id {
        Some(v) => v,
        None => {
            error!("Notice message was not provided with room_id");
            return;
        }
    };
    let req = send_message_event::v3::Request::new(&room_id, &next_txn_id, &content)
        .expect("m.room.message serialization must work");
    if let Err(e) = client.send_request(req).await {
        error!("{:?}", e);
    }
}
pub async fn send_plain_text(
    room_id: Option<OwnedRoomId>,
    storage: &mut ResponderStorage,
    message: String,
    client: &MatrixClient,
) {
    let content = RoomMessageEventContent::text_plain(message);
    let next_txn_id = storage.next_txn_id();
    let room_id = match room_id {
        Some(v) => v,
        None => {
            error!("Plain text message was not provided with room_id");
            return;
        }
    };
    let req = send_message_event::v3::Request::new(&room_id, &next_txn_id, &content)
        .expect("m.room.message serialization must work");
    if let Err(e) = client.send_request(req).await {
        error!("Unable to send response due to error {:?}", e);
    }
}

pub async fn send_formatted_text(
    room_id: Option<OwnedRoomId>,
    storage: &mut ResponderStorage,
    message: String,
    formatted_message: Option<String>,
    client: &MatrixClient,
) {
    let content =
        RoomMessageEventContent::text_html(message, formatted_message.unwrap_or_default());
    let next_txn_id = storage.next_txn_id();
    let room_id = match room_id {
        Some(v) => v,
        None => {
            error!("Formatted text message was not provided with room_id");
            return;
        }
    };
    let req = send_message_event::v3::Request::new(&room_id, &next_txn_id, &content)
        .expect("m.room.message serialization must work");
    if let Err(e) = client.send_request(req).await {
        error!("Unable to send response due to error {:?}", e);
    }
}

pub async fn send_formatted_notice(
    room_id: Option<OwnedRoomId>,
    storage: &mut ResponderStorage,
    message: String,
    formatted_message: Option<String>,
    client: &MatrixClient,
) {
    let content =
        RoomMessageEventContent::notice_html(message, formatted_message.unwrap_or_default());
    let next_txn_id = storage.next_txn_id();
    let room_id = match room_id {
        Some(v) => v,
        None => {
            error!("Formatted notice message was not provided with room_id");
            return;
        }
    };
    let req = send_message_event::v3::Request::new(&room_id, &next_txn_id, &content)
        .expect("m.room.message serialization must work");
    if let Err(e) = client.send_request(req).await {
        error!("{:?}", e);
    }
}

pub async fn send_ban_message(
    user: &UserId,
    _reason: Option<String>,
    rooms: HashSet<OwnedRoomId>,
    client: &MatrixClient,
) {
    for room_id in rooms {
        debug!("Banning user {} in room {}...", user, room_id);
        let req = ban_user::v3::Request::new(&room_id, user);
        // TODO: Figure out way to actually include reason
        if let Err(e) = client.send_request(req).await {
            error!("{:?}", e);
        };
    }
}

pub async fn accept_invite(sender: &UserId, room_id: Option<OwnedRoomId>, client: &MatrixClient) {
    let room_id = match room_id {
        Some(v) => v,
        None => {
            error!("Accept invite message was not provided with room_id");
            return;
        }
    };
    info!("Authorized user {} invited me to room {}", sender, room_id);
    let response = client
        .send_request(join_room_by_id::v3::Request::new(&room_id))
        .await;
    match response {
        Ok(_) => info!("Successfully joined room {}", room_id),
        Err(e) => debug!("Unable to join room {} because of error {:?}", room_id, e),
    }
}

/// Will reject an invite and print the user that tried to logs
pub async fn reject_invite(sender: &UserId, room_id: Option<OwnedRoomId>, client: &MatrixClient) {
    let room_id = match room_id {
        Some(v) => v,
        None => {
            error!("Reject invite message was not provided with room_id");
            return;
        }
    };
    let response = client
        .send_request(leave_room::v3::Request::new(&room_id))
        .await;
    match response {
        Ok(_) => info!("Rejected invite from unathorized user {}", sender),
        Err(e) => debug!("Unable to reject invite this loop because of error {:?}", e),
    }
}
