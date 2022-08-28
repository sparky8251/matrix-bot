use crate::config::MatrixListenerConfig;
use crate::messages::{MatrixBanMessage, MatrixMessage, MatrixMessageType};
use ruma::{events::room::message::TextMessageEventContent, OwnedUserId, UserId};
use tokio::sync::mpsc::Sender;
use tracing::{debug, error, trace};

pub(super) async fn ban_handler(
    text: &TextMessageEventContent,
    config: &MatrixListenerConfig,
    sender: &UserId,
    send: &mut Sender<MatrixMessage>,
) {
    if config.ban_rooms.is_empty() {
        trace!("No rooms specified, ban feature is disabled. Skipping...");
        return;
    }

    if !config.admins.contains(sender) {
        debug!("Unauthorized user for banning. Skipping...");
        return;
    }

    let user: OwnedUserId = match text.body.split(' ').nth(1) {
        Some(v) => match UserId::parse(v) {
            Ok(u) => u,
            Err(_) => {
                error!("User was invalid format, unable to continue to ban handler");
                return;
            }
        },
        None => {
            error!("Ban command doesnt appear to include user, unable to continue");
            return;
        }
    };

    let reason: String = text
        .body
        .split(' ')
        .skip(2)
        .map(|s| s.to_owned() + " ")
        .collect();

    let reason = {
        if reason.trim().is_empty() {
            None
        } else {
            Some(reason)
        }
    };

    let message = MatrixBanMessage {
        user,
        reason,
        rooms: config.ban_rooms.clone(),
    };

    if send
        .send(MatrixMessage {
            room_id: None,
            message: MatrixMessageType::Ban(message),
        })
        .await
        .is_err()
    {
        error!("Channel closed, unable to send mesage.")
    }
}
