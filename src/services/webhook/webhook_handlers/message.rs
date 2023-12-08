use crate::helpers::MatrixFormattedTextResponse;
use crate::messages::{MatrixMessage, MatrixMessageType};
use crate::services::webhook::listener::WebhookListener;
use axum::{
    extract::{Extension, FromRequestParts},
    http::{request::Parts as RequestParts, StatusCode},
    Json,
};
use ruma::{events::room::message::RoomMessageEventContent, OwnedRoomId, OwnedUserId};
use serde::Deserialize;
use std::collections::HashSet;
use std::sync::Arc;

pub async fn message(
    req_token: MessageToken,
    Extension(state): Extension<Arc<WebhookListener>>,
    Json(message): Json<Message>,
) -> StatusCode {
    if req_token.0.eq(&state.config.token) {
        let matrix_message = MatrixMessage {
            room_id: Some(message.room_id.clone()),
            message: MatrixMessageType::Response(RoomMessageEventContent::notice_plain(
                message.message.clone(),
            )),
        };
        if let Err(_) = state.send.clone().send(matrix_message).await {
            return StatusCode::INTERNAL_SERVER_ERROR;
        };
        if let Some(pings) = &message.ping {
            let mut response = MatrixFormattedTextResponse::default();
            let pings: HashSet<OwnedUserId> = pings.iter().cloned().collect();
            response.set_users(pings);
            let formatted_text = response.format_text().unwrap();
            let matrix_message = MatrixMessage {
                room_id: Some(message.room_id.clone()),
                message: MatrixMessageType::Response(RoomMessageEventContent::text_html(
                    response.to_string(),
                    formatted_text,
                )),
            };
            if let Err(_) = state.send.clone().send(matrix_message).await {
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
impl<S> FromRequestParts<S> for MessageToken {
    type Rejection = StatusCode;

    async fn from_request_parts(
        req: &mut RequestParts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        if req.headers.contains_key("X-Webhook-Token") {
            let token = req.headers.get("X-Webhook-Token").unwrap();
            match token.to_str() {
                Ok(v) => Ok(MessageToken(v.to_owned())),
                Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        } else {
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}
