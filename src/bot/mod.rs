use crate::config::{BotConfig, Storage};
use crate::handlers::handle_text_message;

use std::process;
use std::time::Duration;

use log::{debug, error, info, trace};
use ruma_client::{
    api::r0::{
        membership::{join_room_by_id, leave_room},
        sync::sync_events::{self, SetPresence},
    },
    events::{
        collections::all::RoomEvent, room::message::MessageEventContent,
        stripped::AnyStrippedStateEvent,
    },
    Client,
};

pub struct Bot {
    pub storage: Storage,
    pub config: BotConfig,
    pub api_client: reqwest::Client,
}

impl Bot {
    pub fn new(storage: Storage, config: BotConfig, api_client: reqwest::Client) -> Self {
        Self {
            storage,
            config,
            api_client,
        }
    }

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
                                Ok(r) => match r {
                                    RoomEvent::RoomMessage(m) => match m.content {
                                        MessageEventContent::Text(t) => {
                                            match handle_text_message(
                                                &t, &m.sender, room_id, &client, &mut self.storage, &self.config,
                                                &self.api_client,
                                            )
                                            .await
                                            {
                                                Ok(_) => {
                                                    trace!("Handled text message");
                                                    ()
                                                }
                                                Err(e) => {
                                                    debug!("{:?}", e);
                                                    ()
                                                }
                                            }
                                        }
                                        _ => (),
                                    },
                                    _ => (),
                                },
                                Err(e) => {
                                    debug!("{:?}", e);
                                    trace!("Content: {:?}", raw_event.json());
                                    ()
                                }
                            }
                            self.storage.last_sync = Some(v.next_batch.clone());
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
                                        if self.config.admins.contains(&s.sender) {
                                            info!(
                                                "Authorized user {} invited me to room {}",
                                                &s.sender, &room_id
                                            );
                                            let response = client
                                                .request(join_room_by_id::Request {
                                                    room_id: room_id.clone(),
                                                    third_party_signed: None,
                                                })
                                                .await;
                                            match response {
                                                Ok(_) => {
                                                    info!("Successfully joined room {}", &room_id);
                                                    ()
                                                }
                                                Err(e) => {
                                                    debug!(
                                                    "Unable to join room {} because of error {:?}",
                                                    &room_id, e
                                                );
                                                    ()
                                                }
                                            }
                                        } else {
                                            let response = client
                                                .request(leave_room::Request {
                                                    room_id: room_id.clone(),
                                                })
                                                .await;
                                            match response {
                                                Ok(_) => {
                                                    info!(
                                                        "Rejected invite from unathorized user {}",
                                                        s.sender
                                                    );
                                                    ()
                                                }
                                                Err(e) => {
                                                    debug!("Unable to reject invite this loop because of error {:?}", e);
                                                    ()
                                                }
                                            }
                                        }
                                    }
                                    _ => (), //FIXME: Reject invite if there is no known sender
                                },
                                Err(e) => {
                                    debug!("{:?}", e);
                                    trace!("Content: {:?}", raw_event.json());
                                    ()
                                }
                            }
                        }
                    }
                }
                None => {
                    debug!("Response deserialization failed. Doing nothing this loop.");
                    ()
                }
            }
        }
    }
}
