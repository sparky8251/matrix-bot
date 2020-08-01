//! Structs and functions that represent functional bots and allow for easy loading
//! plus main loop initialization.

use crate::config::ResponderStorage;
use crate::matrix_handlers::responders::{
    accept_invite, reject_invite, send_formatted_text, send_notice, send_plain_text,
};
use crate::messages::{MatrixInviteType, MatrixMessage, MatrixMessageType};
use ruma_client::HttpsClient;
use slog::{info, Logger};
use tokio::sync::mpsc::Receiver;

/// Struct representing all required data for a functioning bot instance.
pub struct MatrixResponder {
    /// Storage data.
    pub storage: ResponderStorage,
    pub logger: Logger,
    recv: Receiver<MatrixMessage>,
}

impl MatrixResponder {
    /// Loads storage data, config data, and then creates a reqwest client and then returns a Bot instance.
    pub fn new(logger: &Logger, recv: Receiver<MatrixMessage>) -> Self {
        let storage = ResponderStorage::load_storage(&logger);
        Self {
            storage,
            logger: logger.clone(),
            recv,
        }
    }

    /// Used to start main program loop for the bot.
    /// Will login then loop forever while waiting on new sync data from the homeserver.
    pub async fn start(&mut self, client: HttpsClient) {
        loop {
            match self.recv.recv().await {
                Some(v) => match v.message {
                    MatrixMessageType::Notice(m) => {
                        send_notice(&client, v.room_id, &mut self.storage, m, &self.logger).await
                    }
                    MatrixMessageType::FormattedText(m) => {
                        send_formatted_text(
                            &client,
                            v.room_id,
                            &mut self.storage,
                            m.plain_text,
                            m.formatted_text,
                            &self.logger,
                        )
                        .await
                    }
                    MatrixMessageType::PlainText(m) => {
                        send_plain_text(&client, v.room_id, &mut self.storage, m, &self.logger)
                            .await
                    }
                    MatrixMessageType::Invite(m) => match m.kind {
                        MatrixInviteType::Accept => {
                            accept_invite(m.sender, v.room_id, &client, &self.logger).await
                        }
                        MatrixInviteType::Reject => {
                            reject_invite(m.sender, v.room_id, &client, &self.logger).await
                        }
                    },
                },
                None => {
                    info!(
                        self.logger,
                        "Matrix channel closed and empty. Exiting thread."
                    );
                    break;
                }
            }
        }
    }
}
