// use crate::config::WebhookListenerConfig;
// use crate::helpers::MatrixFormattedTextResponse;
use crate::messages::MatrixMessage;
// use crate::messages::{MatrixFormattedMessage, MatrixMessage, MatrixMessageType};
use rocket::http::Status;
// use rocket::request::{self, FromRequest, Request};
use rocket::State;
use rocket_contrib::json::Json;
// use ruma::{RoomId, UserId};
use serde::Deserialize;
// use serde::{Deserialize, Serialize};
// use std::collections::HashSet;
use tokio::sync::mpsc::Sender as TokioSender;

#[post("/", data = "<event>")]
pub async fn event (
    // req_token: MessageToken,
    event: Json<String>,
    // send: State<'_, TokioSender<MatrixMessage>>,
) -> Status {
    println!("{:?}", event);
    Status::Ok
}