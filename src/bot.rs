use crate::handlers::handle_text_message;
use crate::session::SavedSession;

use std::process;
use std::time::Duration;

use anyhow::Result;
use log::{error, info};
use ruma_client::{
    api::r0::sync::sync_events::{self, SetPresence},
    events::{collections::all::RoomEvent, room::message::MessageEventContent},
    Client,
};
use url::Url;

pub async fn start(homeserver_url: Url, session: &mut SavedSession) {
    match bot(homeserver_url, session).await {
        Ok(v) => info!("{:?}", v),
        Err(e) => {
            error!("{:?}", e);
            // process::exit(32);
        }
    }
}

async fn bot(homeserver_url: Url, session: &mut SavedSession) -> Result<()> {
    let client = Client::https(homeserver_url.clone(), session.get_session());

    if session.get_session().is_none() {
        info!("No previous session found. Creating new session...");
        if session.get_username().is_empty() || session.get_password().is_empty() {
            info!("No username or password found. Writing sample ron file. Please fill out username and password and try again.");
            match session.save_session() {
                Ok(()) => process::exit(12),
                Err(e) => {
                    error!("{:?}", e);
                    process::exit(24)
                }
            }
        } else {
            session.set_session(
                client
                    .log_in(session.get_username(), session.get_password(), None, None)
                    .await?,
            );
            match session.save_session() {
                Ok(()) => (),
                Err(e) => {
                    error!("{:?}", e);
                    process::exit(24)
                }
            };
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
                full_state: false,
                set_presence: SetPresence::Unavailable,
                timeout: Some(Duration::new(1000, 0)),
            })
            .await?;

        for (room_id, joined_room) in &response.rooms.join {
            for raw_event in &joined_room.timeline.events {
                let event = raw_event.deserialize();
                match event {
                    Ok(r) => match r {
                        RoomEvent::RoomMessage(m) => match m.content {
                            MessageEventContent::Text(t) => {
                                handle_text_message(&t, &m.sender, room_id, &client, session)
                                    .await?
                            }
                            _ => (),
                        },
                        _ => (),
                    },
                    Err(e) => {
                        error!("{:?}", e);
                        error!("Content: {:?}", raw_event.json())
                    },
                }
            }
        }

        session.set_last_sync(Some(response.next_batch));
        match session.save_session() {
            Ok(()) => (),
            Err(e) => {
                error!("{:?}", e);
                process::exit(24)
            }
        };
    }
}
