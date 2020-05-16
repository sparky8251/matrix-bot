use log::{debug, info};
use ruma_client::{
    api::r0::membership::leave_room,
    identifiers::{RoomId, UserId},
    HttpsClient,
};

pub async fn reject_invite(sender: &UserId, room_id: &RoomId, client: &HttpsClient) {
    let response = client
        .request(leave_room::Request {
            room_id: room_id.clone(),
        })
        .await;
    match response {
        Ok(_) => {
            info!("Rejected invite from unathorized user {}", &sender);
        }
        Err(e) => {
            debug!("Unable to reject invite this loop because of error {:?}", e);
        }
    }
}
