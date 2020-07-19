//! Simple function for accepting invites

use ruma_client::{
    api::r0::membership::join_room_by_id,
    identifiers::{RoomId, UserId},
    HttpsClient,
};
use slog::{debug, info, Logger};

/// Will accept an invite and print the user and room it has been invited to to console
pub async fn accept_invite(
    sender: &UserId,
    room_id: &RoomId,
    client: &HttpsClient,
    logger: &Logger,
) {
    info!(
        logger,
        "Authorized user {} invited me to room {}", &sender, &room_id
    );
    let response = client
        .request(join_room_by_id::Request {
            room_id: room_id.clone(),
            third_party_signed: None,
        })
        .await;
    match response {
        Ok(_) => info!(logger, "Successfully joined room {}", &room_id),
        Err(e) => debug!(
            logger,
            "Unable to join room {} because of error {:?}", &room_id, e
        ),
    }
}
