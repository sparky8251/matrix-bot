use crate::config::MatrixListenerConfig;
use crate::messages::{MatrixBanMessage, MatrixMessage, MatrixMessageType};
use crate::regex::FORMATTED_USERNAME;
use anyhow::bail;
use ruma::{events::room::message::TextMessageEventContent, OwnedUserId, UserId};
use tokio::sync::mpsc::Sender;
use tracing::{debug, trace, warn};

pub(super) async fn ban_handler(
    text: &TextMessageEventContent,
    config: &MatrixListenerConfig,
    sender: &UserId,
    send: &mut Sender<MatrixMessage>,
) -> anyhow::Result<()> {
    if config.ban_rooms.is_empty() {
        trace!("No rooms specified, ban feature is disabled. Skipping...");
        return Ok(());
    }

    if !config.admins.contains(sender) {
        debug!("Unauthorized user for banning. Skipping...");
        return Ok(());
    }

    trace!("Body text is: {:?}", text.body);
    let user: OwnedUserId = match text.body.split(' ').nth(1) {
        Some(v) => {
            trace!("Body split text: {:?}", v);
            trace!("Attempting parse of plain body text");
            match UserId::parse(v) {
                Ok(u) => u,
                Err(_) => {
                    debug!("Ban command doesnt appear to include user, attempting formatted body parsing");
                    trace!("Formatted body text is: {:?}", text.formatted);
                    match &text.formatted {
                        Some(t) => {
                            let username = match FORMATTED_USERNAME.captures_iter(&t.body).next() {
                                Some(l) => {
                                    trace!(
                                        "Captured username from formatted body pre-parsing: {:?}",
                                        l
                                    );
                                    match UserId::parse(&l[0]) {
                                        Ok(u) => u,
                                        Err(_) => {
                                            bail!("User was invalid format, unable to continue ban handler");
                                        }
                                    }
                                }
                                None => {
                                    bail!("Unable to find HTTPS line in formatted body for user ban. Unable to continue.");
                                }
                            };
                            username
                        }
                        None => {
                            warn!("No formatted body present, unable to attempt parse of user for ban. Unable to continue.");
                            return Ok(());
                        }
                    }
                }
            }
        }
        None => {
            bail!("somehow no body or formatted body was presented?");
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
        bail!("Channel closed, unable to send mesage.");
    }
    Ok(())
}
