//! Structs and functions that represent functional bots and allow for easy loading
//! plus main loop initialization.

use super::MatrixClient;
use crate::config::ResponderStorage;
use crate::matrix_handlers::responders::{
    accept_invite, reject_invite, send_ban_message, send_formatted_notice, send_formatted_text,
    send_notice, send_plain_text,
};
use crate::messages::{MatrixInviteType, MatrixMessage, MatrixMessageType};
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
                    match m {
                        Some(v) => match v.message {
                    MatrixMessageType::Notice(m) => {
                        if let Err(e) = send_notice(&client, v.room_id, &mut self.storage, m).await
                        {
                            error!("{}", e);
                        }
                    }
                    MatrixMessageType::FormattedText(m) => {
                        if let Err(e) = send_formatted_text(
                            v.room_id,
                            &mut self.storage,
                            m.plain_text,
                            m.formatted_text,
                            &client,
                        )
                        .await
                        {
                            error!("{}", e);
                        }
                    }
                    MatrixMessageType::Text(m) => {
                        if let Err(e) =
                            send_plain_text(v.room_id, &mut self.storage, m, &client).await
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
                    MatrixMessageType::FormattedNotice(m) => {
                        if let Err(e) = send_formatted_notice(
                            v.room_id,
                            &mut self.storage,
                            m.plain_text,
                            m.formatted_text,
                            &client,
                        )
                        .await
                        {
                            error!("{}", e);
                        }
                    }
                    MatrixMessageType::Ban(m) => {
                        if let Err(e) = send_ban_message(&m.user, m.reason, m.rooms, &client).await
                        {
                            error!("{}", e);
                        }
                    }
                },
                None => {
                    info!("Matrix channel closed and empty. Exiting thread.");
                    break;
                }
                    }
                }
            }
        }
        trace!("Matrix responder shutdown complete")
    }
}
