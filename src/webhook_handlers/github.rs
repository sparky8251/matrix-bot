// use crate::config::WebhookListenerConfig;
// use crate::helpers::MatrixFormattedTextResponse;
// use crate::messages::MatrixMessage;
// use crate::messages::{MatrixFormattedMessage, MatrixMessage, MatrixMessageType};
use rocket::http::Status;
// use rocket::request::{self, FromRequest, Request};
// use rocket::State;
use rocket_contrib::json::Json;
// use ruma::{RoomId, UserId};
// use serde::Deserialize;
// use serde::{Deserialize, Serialize};
// use std::collections::HashSet;
// use tokio::sync::mpsc::Sender as TokioSender;
use crate::events::Event;
use tracing::{debug, error, trace};

#[post("/", data = "<event>")]
pub async fn event(
    // req_token: MessageToken,
    event: Json<Event>, // send: State<'_, TokioSender<MatrixMessage>>,
) -> Status {
    match event.clone() {
        Event::Release {
            release,
            repository,
            ..
        } => {
            let url = release.html_url;
            let repo_name = repository.name;
            let release_name = match release.name {
                Some(v) => v,
                None => {
                    error!(
                        "No release name has been set. Unable to announce release for event {:?}",
                        event
                    );
                    return Status::PreconditionFailed;
                }
            };
            let prerelease = if release.prerelease {
                "pre".to_string()
            } else {
                String::new()
            };
            let body = match release.body {
                Some(v) => v,
                None => {
                    error!(
                        "No release body has been set. Unable to annouce release for event {:?}",
                        event
                    );
                    return Status::PreconditionFailed;
                }
            };

            let message = format!(
                "A new {}release has been made for {}! {} is ready for using. Read more here: {}",
                prerelease, repo_name, release_name, url
            );
            debug!("{}", message);
            trace!("{}", body);
            Status::Ok
        }
        _ => Status::UnprocessableEntity,
    }
}
