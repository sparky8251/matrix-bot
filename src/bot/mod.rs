//! Structs and functions that represent functional bots and allow for easy loading
//! plus main loop initialization.

use crate::config::{Config, Storage};
use crate::handlers::{handle_invite_event, handle_text_event};

use std::process;
use std::time::Duration;

use log::{debug, error, info, trace};
use ruma_client::{
    api::r0::sync::sync_events::{self, SetPresence},
    events::{
        collections::all::RoomEvent, room::message::MessageEventContent,
        stripped::AnyStrippedStateEvent,
    },
    Client,
};

/// Struct representing all required data for a functioning bot instance.
pub struct Bot {
    /// Storage data.
    pub storage: Storage,
    /// Configuration data.
    pub config: Config,
    /// Reqwest client used for external API calls.
    pub api_client: reqwest::Client,
}

impl Bot {
    /// Loads storage data, config data, and then creates a reqwest client and then returns a Bot instance.
    pub fn new() -> Self {
        let storage = Storage::load_storage();
        let config = Config::load_bot_config();
        let api_client = reqwest::Client::new();
        Self {
            storage,
            config,
            api_client,
        }
    }

    /// Used to start main program loop for the bot.
    /// Will login then loop forever while waiting on new sync data from the homeserver.
    pub async fn start(&mut self) {
        let client = Client::https(self.config.mx_url.clone(), self.storage.session.clone());
        self.storage.session = match client
            .log_in(
                self.config.mx_uname.localpart().to_string(),
                self.config.mx_pass.clone(),
                None,
                None,
            )
            .await
        {
            Ok(v) => Some(v),
            Err(e) => {
                error!(
                    "Unable to login as {} on {} due to error {:?}",
                    self.config.mx_uname.localpart(),
                    self.config.mx_url,
                    e
                );
                process::exit(8)
            }
        };
        trace!("Session retrived, saving storage data...");
        self.storage.save_storage();
        info!("Successfully logged in as {}", self.config.mx_uname);

        loop {
            let response = match client
                .request(sync_events::Request {
                    filter: None,
                    since: self.storage.last_sync.clone(),
                    full_state: false,
                    set_presence: SetPresence::Unavailable,
                    timeout: Some(Duration::new(2000, 0)),
                })
                .await
            {
                Ok(v) => Some(v),
                Err(e) => {
                    debug!("Line 73: {:?}", e);
                    None
                }
            };

            match response {
                Some(v) => {
                    for (room_id, joined_room) in &v.rooms.join {
                        for raw_event in &joined_room.timeline.events {
                            let event = raw_event.deserialize();
                            match event {
                                Ok(r) => {
                                    if let RoomEvent::RoomMessage(m) = r {
                                        if let MessageEventContent::Text(t) = m.content {
                                            handle_text_event(
                                                &t,
                                                &m.sender,
                                                room_id,
                                                &client,
                                                &mut self.storage,
                                                &self.config,
                                                &self.api_client,
                                            )
                                            .await;
                                        }
                                    }
                                }
                                Err(e) => {
                                    debug!("{:?}", e);
                                    trace!("Content: {:?}", raw_event.json())
                                }
                            }
                            self.storage.last_sync = Some(v.next_batch.clone());
                            self.storage.save_storage();
                        }
                    }
                    for (room_id, invited_room) in &v.rooms.invite {
                        trace!("Invited room data: {:?}", invited_room);
                        for raw_event in &invited_room.invite_state.events {
                            let event = raw_event.deserialize();
                            match event {
                                Ok(v) => match v {
                                    AnyStrippedStateEvent::RoomMember(s) => {
                                        trace!("Invited by {}", s.sender);
                                        handle_invite_event(
                                            &s.sender,
                                            &room_id,
                                            &client,
                                            &self.config,
                                        )
                                        .await;
                                        trace!("Handled invite event")
                                    }
                                    _ => error!("No known inviter. Will not join room. If you see this, report it."), //FIXME: Reject invite if there is no known sender
                                },
                                Err(e) => {
                                    debug!("{:?}", e);
                                    trace!("Content: {:?}", raw_event.json())
                                }
                            }
                        }
                    }
                }
                None => debug!("Response deserialization failed. Doing nothing this loop."),
            }
        }
    }
}
