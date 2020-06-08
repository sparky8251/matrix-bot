//! Simple function for accepting invites

use log::{debug, info};
use ruma_client::{
    api::r0::membership::join_room_by_id,
    identifiers::{RoomId, UserId},
    HttpsClient,
};

/// Will accept an invite and print the user and room it has been invited to to console
pub async fn accept_invite(sender: &UserId, room_id: &RoomId, client: &HttpsClient) {
    info!(
        "Authorized user {} invited me to room {}",
        &sender, &room_id
    );
    let response = client
        .request(join_room_by_id::Request {
            room_id: room_id.clone(),
            third_party_signed: None,
        })
        .await;
    match response {
        Ok(_) => info!("Successfully joined room {}", &room_id),
        Err(e) => debug!("Unable to join room {} because of error {:?}", &room_id, e),
    }
}
