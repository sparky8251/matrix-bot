use crate::config::ResponderStorage;
use crate::matrix::MatrixClient;
use anyhow::Context;
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
) -> anyhow::Result<()> {
    let content = RoomMessageEventContent::notice_plain(message);
    let next_txn_id = storage.next_txn_id();
    let room_id = room_id.context("Notice message was not provided with room_id")?;

    let req = send_message_event::v3::Request::new(&room_id, &next_txn_id, &content)
        .context("m.room.message serialization must work")?;

    client
        .send_request(req)
        .await
        .context("Matrix response was unable to be sent")?;
    Ok(())
}
pub async fn send_plain_text(
    room_id: Option<OwnedRoomId>,
    storage: &mut ResponderStorage,
    message: String,
    client: &MatrixClient,
) -> anyhow::Result<()> {
    let content = RoomMessageEventContent::text_plain(message);
    let next_txn_id = storage.next_txn_id();
    let room_id = room_id.context("Plain text message was not provided with a room_id")?;
    let req = send_message_event::v3::Request::new(&room_id, &next_txn_id, &content)
        .context("m.room.message.serialziation must work")?;
    client
        .send_request(req)
        .await
        .context("Matrix response was unable to be sent")?;
    Ok(())
}

pub async fn send_formatted_text(
    room_id: Option<OwnedRoomId>,
    storage: &mut ResponderStorage,
    message: String,
    formatted_message: Option<String>,
    client: &MatrixClient,
) -> anyhow::Result<()> {
    let content =
        RoomMessageEventContent::text_html(message, formatted_message.unwrap_or_default());
    let next_txn_id = storage.next_txn_id();
    let room_id = room_id.context("Formatted text message was not provided with room_id")?;
    let req = send_message_event::v3::Request::new(&room_id, &next_txn_id, &content)
        .context("m.room.message serialization must work")?;
    client
        .send_request(req)
        .await
        .context("Matrix response was unable to be sent")?;
    Ok(())
}

pub async fn send_formatted_notice(
    room_id: Option<OwnedRoomId>,
    storage: &mut ResponderStorage,
    message: String,
    formatted_message: Option<String>,
    client: &MatrixClient,
) -> anyhow::Result<()> {
    let content =
        RoomMessageEventContent::notice_html(message, formatted_message.unwrap_or_default());
    let next_txn_id = storage.next_txn_id();
    let room_id = room_id.context("Formatted notice message was not provided with room_id")?;
    let req = send_message_event::v3::Request::new(&room_id, &next_txn_id, &content)
        .context("m.room.message serialization must work")?;
    client
        .send_request(req)
        .await
        .context("Matrix response was unable to be sent")?;
    Ok(())
}

pub async fn send_ban_message(
    user: &UserId,
    reason: Option<String>,
    rooms: HashSet<OwnedRoomId>,
    client: &MatrixClient,
) -> anyhow::Result<()> {
    for room_id in rooms {
        debug!("Banning user {} in room {}...", user, room_id);
        let mut req = ban_user::v3::Request::new(&room_id, user);
        req.reason = reason.as_deref();
        if let Err(e) = client.send_request(req).await {
            error!("{:?}", e);
        };
    }
    Ok(())
}

pub async fn accept_invite(
    sender: &UserId,
    room_id: Option<OwnedRoomId>,
    client: &MatrixClient,
) -> anyhow::Result<()> {
    let room_id = room_id.context("Accept invite message was not provided with room_id")?;
    info!("Authorized user {} invited me to room {}", sender, room_id);
    client
        .send_request(join_room_by_id::v3::Request::new(&room_id))
        .await
        .context("Unable to join room")?;

    info!("Successfully joined room {}", room_id);
    Ok(())
}

/// Will reject an invite and print the user that tried to logs
pub async fn reject_invite(
    sender: &UserId,
    room_id: Option<OwnedRoomId>,
    client: &MatrixClient,
) -> anyhow::Result<()> {
    let room_id = room_id.context("Reject invite message was not provided with room_id")?;
    client
        .send_request(leave_room::v3::Request::new(&room_id))
        .await
        .context("Unable to reject invite")?;

    info!("Rejected invite from unathorized user {}", sender);
    Ok(())
}
