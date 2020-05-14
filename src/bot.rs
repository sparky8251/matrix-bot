use crate::config::{BotConfig, Storage};
use crate::handlers::handle_text_message;

use std::process;
use std::time::Duration;

use anyhow::Result;
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

pub async fn start(storage: &mut Storage, config: &BotConfig, api_client: &reqwest::Client) {
    match bot(storage, config, api_client).await {
        Ok(v) => debug!("{:?}", v),
        Err(e) => {
            debug!("{:?}", e);
        }
    }
}

async fn bot(
    storage: &mut Storage,
    config: &BotConfig,
    api_client: &reqwest::Client,
) -> Result<()> {
    let client = Client::https(config.mx_url.clone(), storage.session.clone());
    storage.session = match client
        .log_in(
            config.mx_uname.localpart().to_string(),
            config.mx_pass.clone(),
            None,
            None,
        )
        .await
    {
        Ok(v) => Some(v),
        Err(e) => {
            error!(
                "Unable to login as {} on {} due to error {:?}",
                config.mx_uname.localpart(),
                config.mx_url,
                e
            );
            process::exit(8)
        }
    };
    trace!("Session retrived, saving storage data...");
    storage.save_storage();
    info!("Successfully logged in as {}", config.mx_uname);

    loop {
        let response = match client
            .request(sync_events::Request {
                filter: None,
                since: storage.last_sync.clone(),
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
                                            &t, &m.sender, room_id, &client, storage, config,
                                            api_client,
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
                        storage.last_sync = Some(v.next_batch.clone());
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
                                    if config.admins.contains(&s.sender) {
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
