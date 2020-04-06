use std::process;

use log::info;
use ruma_client::{api::r0::sync::sync_events, Client};
use ruma_events::{
    collections::all::RoomEvent,
    room::message::{MessageEvent, MessageEventContent},
    EventResult,
};
use url::Url;

use crate::error::Result;
use crate::handler::handle_text_message;
use crate::session::SavedSession;

pub async fn start(homeserver_url: Url, session: &mut SavedSession) {
    bot(homeserver_url, session).await.unwrap();
}

async fn bot(homeserver_url: Url, session: &mut SavedSession) -> Result<()> {
    let client = Client::https(homeserver_url.clone(), session.get_session());

    if session.get_session().is_none() {
        info!("No previous session found. Creating new session...");
        if session.get_username().is_empty() || session.get_password().is_empty() {
            info!("No username or password found. Writing sample ron file. Please fill out username and password and try again.");
            session.save_session().unwrap();
            process::exit(1);
        } else {
            session.set_session(
                client
                    .log_in(session.get_username(), session.get_password(), None, None)
                    .await?,
            );
            session.save_session().unwrap();
        }
    }
    info!(
        "Successfully logged in as {} on {}",
        session.get_username(),
        homeserver_url
    );

    loop {
        let response = client
            .request(sync_events::Request {
                filter: None,
                since: session.get_last_sync(),
                full_state: None,
                set_presence: None,
                timeout: Some(1000_u32.into()),
            })
            .await?;

        for (room_id, joined_room) in &response.rooms.join {
            for event in &joined_room.timeline.events {
                if let EventResult::Ok(RoomEvent::RoomMessage(MessageEvent {
                    content: MessageEventContent::Text(text),
                    sender,
                    ..
                })) = event
                {
                    handle_text_message(&text, sender, room_id, &client, session).await?;
                }
            }
        }

        session.set_last_sync(Some(response.next_batch));
        session.save_session().unwrap();
    }
}
