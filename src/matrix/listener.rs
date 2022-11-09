//! Structs and functions that represent functional bots and allow for easy loading
//! plus main loop initialization.

use super::MatrixClient;
use crate::config::{Config, ListenerStorage, MatrixListenerConfig};
use crate::matrix_handlers::listeners::{handle_invite_event, handle_text_event};
use crate::messages::MatrixMessage;
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
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tokio::sync::watch::Receiver;
use tracing::{debug, error, trace};

/// Struct representing all required data for a functioning bot instance.
pub struct MatrixListener {
    /// Storage data.
    pub storage: ListenerStorage,
    /// Configuration data.
    pub config: MatrixListenerConfig,
    /// Reqwest client used for external API calls.
    pub api_client: reqwest::Client,
    send: Sender<MatrixMessage>,
}

impl MatrixListener {
    /// Loads storage data, config data, and then creates a reqwest client and then returns a Bot instance.
    pub fn new(config: &Config, send: Sender<MatrixMessage>) -> anyhow::Result<Self> {
        let storage = ListenerStorage::load_storage()?;
        let config = MatrixListenerConfig::new(config);
        let api_client = reqwest::Client::new();
        Ok(Self {
            storage,
            config,
            api_client,
            send,
        })
    }

    /// Used to start main program loop for the bot.
    /// Will login then loop forever while waiting on new sync data from the homeserver.
    pub async fn start(&mut self, client: MatrixClient, shutdown_rx: Receiver<bool>) {
        loop {
            if *shutdown_rx.borrow() {
                trace!("Received shutdown on matrix listenener thread");
                break;
            }
            let mut req = sync_events::v3::Request::new();
            req.filter = None;
            req.since = match &self.storage.last_sync {
                Some(v) => Some(v.as_str()),
                None => None,
            };
            req.full_state = false;
            req.set_presence = &PresenceState::Unavailable;
            req.timeout = Some(Duration::new(30, 0));

            let response = match client.send_request(req).await {
                Ok(v) => Some(v),
                Err(e) => {
                    debug!("Line 65: {:?}", e);
                    None
                }
            };

            match response {
                Some(v) => {
                    self.storage.last_sync = Some(v.next_batch.clone());
                    if let Err(e) = self.storage.save_storage() {
                        error!(
                            "Unable to save matrix_listener.ron during normal operation. {}",
                            e
                        )
                    };
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
        trace!("Matrix listener shutdown complete")
    }
}
