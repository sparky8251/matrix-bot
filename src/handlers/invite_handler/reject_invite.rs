//! Simple function for rejecting invites

use ruma_client::{
    api::r0::membership::leave_room,
    identifiers::{RoomId, UserId},
    HttpsClient,
};
use slog::{debug, info, Logger};

/// Will reject an invite and print the user that tried to logs
pub async fn reject_invite(
    sender: &UserId,
    room_id: &RoomId,
    client: &HttpsClient,
    logger: &Logger,
) {
    let response = client
        .request(leave_room::Request {
            room_id: room_id.clone(),
        })
        .await;
    match response {
        Ok(_) => info!(logger, "Rejected invite from unathorized user {}", &sender),
        Err(e) => debug!(
            logger,
            "Unable to reject invite this loop because of error {:?}", e
        ),
    }
}
