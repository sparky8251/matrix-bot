//! Structs and functions that represent functional bots and allow for easy loading
//! plus main loop initialization.

use super::MatrixClient;
use crate::config::{Config, MatrixListenerConfig};
use crate::messages::MatrixMessage;
use crate::services::matrix::matrix_handlers::listeners::{handle_invite_event, handle_text_event};
use ruma::{
    api::client::sync::sync_events,
    events::{
        room::message::{
            MessageType, OriginalSyncRoomMessageEvent, Relation, RoomMessageEventContent,
            SyncRoomMessageEvent,
        },
        AnyStrippedStateEvent, AnySyncMessageLikeEvent, AnySyncTimelineEvent,
    },
    presence::PresenceState,
};
use sled::Tree;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tokio::sync::watch::Receiver;
use tracing::{debug, error, trace};

/// Struct representing all required data for a functioning bot instance.
pub struct MatrixListener {
    /// Configuration data.
    pub config: MatrixListenerConfig,
    /// Reqwest client used for external API calls.
    pub api_client: reqwest::Client,
    send: Sender<MatrixMessage>,
    /// Storage data.
    pub storage: Tree,
}

impl MatrixListener {
    /// Loads storage data, config data, and then creates a reqwest client and then returns a Bot instance.
    pub fn new(
        config: &Config,
        send: Sender<MatrixMessage>,
        storage: Tree,
    ) -> anyhow::Result<Self> {
        let config = MatrixListenerConfig::new(config);
        let api_client = reqwest::Client::new();
        Ok(Self {
            config,
            api_client,
            send,
            storage,
        })
    }

    /// Used to start main program loop for the bot.
    /// Will login then loop forever while waiting on new sync data from the homeserver.
    pub async fn start(&mut self, client: MatrixClient, mut shutdown_rx: Receiver<bool>) {
        loop {
            let mut req = sync_events::v3::Request::new();
            req.filter = None;
            let last_sync = self
                .storage
                .get("last_sync")
                .unwrap()
                .map(|s| String::from_utf8(s.to_vec()).unwrap());
            req.since = last_sync.as_deref();
            req.full_state = false;
            req.set_presence = &PresenceState::Unavailable;
            req.timeout = Some(Duration::new(30, 0));

            tokio::select! {
                _ = shutdown_rx.changed() => {
                    trace!("Received shutdown on matrix listener thread");
                    break;
                },
                response = client.send_request(req) => {
                    let response = match response {
                        Ok(v) => Some(v),
                        Err(e) => {
                            debug!("Line 65: {:?}", e);
                            None
                        }
                    };

                    match response {
                        Some(v) => {
                            let _ = self.storage.insert("last_sync", v.next_batch.as_bytes());
                            self.storage.flush().unwrap();
                            for (room_id, joined_room) in &v.rooms.join {
                                for raw_event in &joined_room.timeline.events {
                                    let event = raw_event.deserialize();
                                    match event {
                                        Ok(AnySyncTimelineEvent::MessageLike(
                                            AnySyncMessageLikeEvent::RoomMessage(
                                                SyncRoomMessageEvent::Original(
                                                    OriginalSyncRoomMessageEvent {
                                                        content:
                                                            RoomMessageEventContent {
                                                                msgtype: MessageType::Text(t),
                                                                relates_to,
                                                                ..
                                                            },
                                                        sender,
                                                        ..
                                                    },
                                                ),
                                            ),
                                        )) => {
                                            if matches!(relates_to, Some(Relation::Replacement(_))) {
                                                debug!("Message is an edit, skipping handling");
                                                continue;
                                            }
                                            if let Err(e) = handle_text_event(
                                                &t,
                                                relates_to.as_ref(),
                                                &sender,
                                                room_id,
                                                &mut self.storage,
                                                &self.config,
                                                &self.api_client,
                                                &mut self.send,
                                            )
                                            .await
                                            {
                                                error!("{}", e);
                                            };
                                        }
                                        Ok(_) => {}
                                        Err(e) => {
                                            debug!("{:?}", e);
                                            trace!("Content: {:?}", raw_event.json())
                                        }
                                    }
                                }
                            }
                            for (room_id, invited_room) in &v.rooms.invite {
                                trace!("Invited room data: {:?}", invited_room);
                                for raw_event in &invited_room.invite_state.events {
                                    let event = raw_event.deserialize();
                                    match event {
                                        Ok(AnyStrippedStateEvent::RoomMember(s)) => {
                                            trace!("Invited by {}", s.sender);
                                            if let Err(e) = handle_invite_event(
                                                &s.sender,
                                                room_id,
                                                &self.config,
                                                &mut self.send,
                                            )
                                            .await
                                            {
                                                error!("{}", e);
                                            };
                                            trace!("Handled invite event")
                                        }
                                        Ok(_) => {
                                            // FIXME: Reject invite if there is no known sender
                                            error!("No known inviter. Will not join room. If you see this, report it.");
                                        }
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
        trace!("Matrix listener shutdown complete")
    }
}
