use crate::services::matrix::MatrixClient;
use anyhow::Context;
use ruma::{
    api::client::{
        membership::{ban_user, join_room_by_id, leave_room},
        message::send_message_event,
    },
    events::room::message::RoomMessageEventContent,
    OwnedRoomId, TransactionId, UserId,
};
use std::collections::HashSet;
use tracing::{debug, error, info};

pub async fn send_message(
    client: &MatrixClient,
    room_id: OwnedRoomId,
    content: RoomMessageEventContent,
) -> anyhow::Result<()> {
    let txn_id = TransactionId::new();
    let req = send_message_event::v3::Request::new(&room_id, &txn_id, &content)
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
