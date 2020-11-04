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

#[post("/", data = "<payload>")]
pub async fn pull_request(
    // req_token: MessageToken,
    payload: Json<PullRequestPayload>,
    _send: State<'_, TokioSender<MatrixMessage>>,
) -> Status {
    println!("{:?}", payload);
    Status::Ok
}

#[derive(Debug, Deserialize)]
pub struct PullRequestPayload {
    action: Option<Actions>,
    number: usize,
    changes: Option<String>,
    pull_request: String,
    repository: String,
    organization: String,
    intallation: String,
    sender: String,
}

#[derive(Debug, Deserialize)]
pub enum Actions {
    #[serde(rename = "opened")]
    Opened,
    #[serde(rename = "edited")]
    Edited,
    #[serde(rename = "closed")]
    Closed,
    #[serde(rename = "assigned")]
    Assigned,
    #[serde(rename = "assigned")]
    Unassigned,
    #[serde(rename = "review_requested")]
    ReviewRequested,
    #[serde(rename = "review_request_removed")]
    ReviewRequestRemoved,
    #[serde(rename = "ready_for_review")]
    ReadyForReview,
    #[serde(rename = "labeled")]
    Labeled,
    #[serde(rename = "unlocked")]
    Unlocked,
    #[serde(rename = "reopened")]
    Reopened,
    #[serde(other)]    
    Unknown,
}