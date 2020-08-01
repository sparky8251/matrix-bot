use crate::config::ResponderStorage;
use ruma_client::{
    api::r0::{
        membership::{join_room_by_id, leave_room},
        message::create_message_event,
    },
    events::{
        room::message::{MessageEventContent, NoticeMessageEventContent, TextMessageEventContent},
        EventJson, EventType,
    },
    identifiers::{RoomId, UserId},
    HttpsClient,
};
use slog::{debug, error, info, Logger};

pub async fn send_notice(
    client: &HttpsClient,
    room_id: RoomId,
    storage: &mut ResponderStorage,
    message: String,
    logger: &Logger,
) {
    match client
        .request(create_message_event::Request {
            room_id: room_id.clone(), //FIXME: There has to be a better way than to clone here
            event_type: EventType::RoomMessage,
            txn_id: storage.next_txn_id(),
            data: EventJson::from(MessageEventContent::Notice(NoticeMessageEventContent {
                body: message,
                relates_to: None,
                format: None,
                formatted_body: None,
            }))
            .into_json(),
        })
        .await
    {
        Ok(_) => (),
        Err(e) => error!(logger, "{:?}", e),
    }
}
pub async fn send_plain_text(
    client: &HttpsClient,
    room_id: RoomId,
    storage: &mut ResponderStorage,
    message: String,
    logger: &Logger,
) {
    match client
        .request(create_message_event::Request {
            room_id: room_id.clone(), //FIXME: There has to be a better way than to clone here
            event_type: EventType::RoomMessage,
            txn_id: storage.next_txn_id(),
            data: EventJson::from(MessageEventContent::Text(TextMessageEventContent {
                body: message,
                format: None,
                formatted_body: None,
                relates_to: None,
            }))
            .into_json(),
        })
        .await
    {
        Ok(_) => (),
        Err(e) => error!(logger, "Unable to send response due to error {:?}", e),
    }
}

pub async fn send_formatted_text(
    client: &HttpsClient,
    room_id: RoomId,
    storage: &mut ResponderStorage,
    message: String,
    formatted_message: Option<String>,
    logger: &Logger,
) {
    match client
        .request(create_message_event::Request {
            room_id: room_id.clone(), //FIXME: There has to be a better way than to clone here
            event_type: EventType::RoomMessage,
            txn_id: storage.next_txn_id(),
            data: EventJson::from(MessageEventContent::Text(TextMessageEventContent {
                body: message,
                format: Some("org.matrix.custom.html".to_string()),
                formatted_body: formatted_message,
                relates_to: None,
            }))
            .into_json(),
        })
        .await
    {
        Ok(_) => (),
        Err(e) => error!(logger, "Unable to send response due to error {:?}", e),
    }
}

pub async fn accept_invite(sender: UserId, room_id: RoomId, client: &HttpsClient, logger: &Logger) {
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

/// Will reject an invite and print the user that tried to logs
pub async fn reject_invite(sender: UserId, room_id: RoomId, client: &HttpsClient, logger: &Logger) {
    let response = client
        .request(leave_room::Request { room_id: room_id })
        .await;
    match response {
        Ok(_) => info!(logger, "Rejected invite from unathorized user {}", &sender),
        Err(e) => debug!(
            logger,
            "Unable to reject invite this loop because of error {:?}", e
        ),
    }
}
