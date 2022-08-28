use crate::helpers::MatrixFormattedTextResponse;
use crate::messages::{MatrixFormattedMessage, MatrixMessage, MatrixMessageType};
use crate::webhook::listener::WebhookListener;
use axum::{
    extract::{Extension, FromRequest, RequestParts},
    http::StatusCode,
    Json,
};
use ruma::{OwnedRoomId, OwnedUserId};
use serde::Deserialize;
use std::collections::HashSet;
use std::sync::Arc;

pub async fn message(
    req_token: MessageToken,
    Json(message): Json<Message>,
    Extension(state): Extension<Arc<WebhookListener>>,
) -> StatusCode {
    if req_token.0.eq(&state.config.token) {
        let matrix_message = MatrixMessage {
            room_id: Some(message.room_id.clone()),
            message: MatrixMessageType::Notice(message.message.clone()),
        };
        if state.send.clone().send(matrix_message).await.is_err() {
            return StatusCode::INTERNAL_SERVER_ERROR;
        };
        if let Some(pings) = &message.ping {
            let mut response = MatrixFormattedTextResponse::default();
            let pings: HashSet<OwnedUserId> = pings.iter().cloned().collect();
            response.set_users(pings);
            let matrix_message = MatrixMessage {
                room_id: Some(message.room_id.clone()),
                message: MatrixMessageType::FormattedText(MatrixFormattedMessage {
                    plain_text: response.to_string(),
                    formatted_text: response.format_text(),
                }),
            };
            if state.send.clone().send(matrix_message).await.is_err() {
                return StatusCode::INTERNAL_SERVER_ERROR;
            };
        };
        StatusCode::OK
    } else {
        StatusCode::UNAUTHORIZED
    }
}

#[derive(Debug, Deserialize)]
pub struct Message {
    room_id: OwnedRoomId,
    message: String,
    ping: Option<Vec<OwnedUserId>>,
}

#[derive(Debug, Deserialize)]
pub struct MessageToken(String);

#[axum::async_trait]
impl<B: std::marker::Send> FromRequest<B> for MessageToken {
    type Rejection = StatusCode;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        if req.headers().contains_key("X-Webhook-Token") {
            let token = req.headers().get("X-Webhook-Token").unwrap();
            match token.to_str() {
                Ok(v) => Ok(MessageToken(v.to_owned())),
                Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        } else {
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}
