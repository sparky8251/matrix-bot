//! Structs and functions that represent functional bots and allow for easy loading
//! plus main loop initialization.

use super::MatrixClient;
use crate::config::ResponderStorage;
use crate::messages::{MatrixInviteType, MatrixMessage, MatrixMessageType};
use crate::services::matrix::matrix_handlers::responders::{
    accept_invite, reject_invite, send_ban_message, send_message,
};
use tokio::sync::{mpsc, watch};
use tracing::{error, info, trace};

/// Struct representing all required data for a functioning bot instance.
pub struct MatrixResponder {
    /// Storage data.
    pub storage: ResponderStorage,
    recv: mpsc::Receiver<MatrixMessage>,
}

impl MatrixResponder {
    /// Loads storage data, config data, and then creates a reqwest client and then returns a Bot instance.
    pub fn new(recv: mpsc::Receiver<MatrixMessage>) -> anyhow::Result<Self> {
        let storage = ResponderStorage::load_storage()?;
        Ok(Self { storage, recv })
    }

    /// Used to start main program loop for the bot.
    /// Will login then loop forever while waiting on new sync data from the homeserver.
    pub async fn start(&mut self, client: MatrixClient, mut shutdown_rx: watch::Receiver<bool>) {
        loop {
            tokio::select! {
                _ = shutdown_rx.changed() => {
                    trace!("Received shutdown on matrix responder thread");
                    break;
                },
                m = self.recv.recv() => {
                    if let Err(e) = self.send_message_handler(m, &client).await {
                        error!("{}", e);
                    }
                }
            }
        }
        trace!("Matrix responder shutdown complete")
    }

    async fn send_message_handler(
        &mut self,
        message: Option<MatrixMessage>,
        client: &MatrixClient,
    ) -> anyhow::Result<()> {
        match message {
            Some(v) => match v.message {
                MatrixMessageType::Response(m) => {
                    if let Err(e) =
                        send_message(&client, v.room_id.unwrap(), &mut self.storage, m).await
                    {
                        error!("{}", e);
                    }
                }
                MatrixMessageType::Invite(m) => match m.kind {
                    MatrixInviteType::Accept => {
                        if let Err(e) = accept_invite(&m.sender, v.room_id, &client).await {
                            error!("{}", e);
                        }
                    }
                    MatrixInviteType::Reject => {
                        if let Err(e) = reject_invite(&m.sender, v.room_id, &client).await {
                            error!("{}", e);
                        }
                    }
                },
                MatrixMessageType::Ban(m) => {
                    if let Err(e) = send_ban_message(&m.user, m.reason, m.rooms, &client).await {
                        error!("{}", e);
                    }
                }
            },
            None => {
                info!("Matrix channel closed and empty. Exiting thread.");
            }
        }
        Ok(())
    }
}
