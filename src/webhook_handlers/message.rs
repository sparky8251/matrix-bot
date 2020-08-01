use crate::config::WebhookListenerConfig;
use crate::messages::{MatrixMessage, MatrixMessageType};
use rocket::http::Status;
use rocket::request::{self, FromRequest, Request};
use rocket::State;
use rocket_contrib::json::Json;
use ruma_client::identifiers::RoomId;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::Sender;

#[post("/", data = "<message>")]
pub async fn message(
    req_token: MessageToken,
    message: Json<Message>,
    conf: State<'_, WebhookListenerConfig>,
    send: State<'_, Sender<MatrixMessage>>,
) -> Status {
    if req_token.0.eq(&conf.token) {
        let matrix_message = MatrixMessage {
            room_id: message.room_id.clone(),
            message: MatrixMessageType::Notice(message.message.clone()),
        };
        match send.clone().send(matrix_message).await {
            Ok(_) => Status::Ok,
            Err(_) => Status::InternalServerError,
        }
    } else {
        Status::Unauthorized
    }
}

#[derive(Debug, Deserialize)]
pub struct Message {
    room_id: RoomId,
    message: String,
}

#[derive(Debug, Deserialize)]
pub struct MessageToken(String);

#[derive(Debug, Serialize)]
pub enum MessageError {
    NoToken,
}

#[rocket::async_trait]
impl<'a, 'r> FromRequest<'a, 'r> for MessageToken {
    type Error = MessageError;

    async fn from_request(req: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        if req.headers().contains("X-Webhook-Token") {
            let token = req.headers().get_one("X-Webhook-Token").unwrap();
            request::Outcome::Success(MessageToken(token.to_string()))
        } else {
            request::Outcome::Failure((Status::Unauthorized, MessageError::NoToken))
        }
    }
}
