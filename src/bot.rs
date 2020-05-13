use crate::handlers::handle_text_message;
use crate::session::{
    save_authorized_users, save_searchable_repos, AuthorizedUsers, SavedSession, SearchableRepos,
};

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
use url::Url;

pub async fn start(
    homeserver_url: Url,
    session: &mut SavedSession,
    authorized_users: &AuthorizedUsers,
    searchable_repos: &SearchableRepos,
    api_client: &reqwest::Client,
) {
    match bot(
        homeserver_url,
        session,
        authorized_users,
        searchable_repos,
        api_client,
    )
    .await
    {
        Ok(v) => debug!("{:?}", v),
        Err(e) => {
            debug!("{:?}", e);
        }
    }
}

async fn bot(
    homeserver_url: Url,
    session: &mut SavedSession,
    authorized_users: &AuthorizedUsers,
    searchable_repos: &SearchableRepos,
    api_client: &reqwest::Client,
) -> Result<()> {
    let client = Client::https(homeserver_url.clone(), session.get_session());
    if authorized_users.get_authorized_users().is_empty() {
        info!("No authorized users found. Must have at least 1 for management functions. Please add at least 1 user in the format of '@sparky:matrix.possumlodge.me' and restart.");
        match save_authorized_users() {
            Ok(_) => process::exit(12),
            Err(e) => {
                error!("Unable to write file due to error {:?}", e);
                process::exit(24)
            }
        }
    }
    if searchable_repos.get_searchable_repos().is_empty() {
        info!("No searchable repos found. If you want to search github, please add at least 1 repo in the format of {{\"jellyfin\":\"/jellyfin/jellyfin\"}} and restart.");
        match save_searchable_repos() {
            Ok(_) => (),
            Err(e) => {
                error!("Unable to write file due to error {:?}", e);
                process::exit(24)
            }
        }
    }
    if session.get_session().is_none() {
        info!("No previous session found. Creating new session...");
        if session.get_username().is_empty() || session.get_password().is_empty() {
            info!("No username or password found. Writing sample ron file. Please fill out username and password and try again.");
            match session.save_session() {
                Ok(()) => process::exit(12),
                Err(e) => {
                    error!("{:?}", e);
                    process::exit(24)
                }
            }
        } else if session.get_gh_username().is_empty() || session.get_gh_password().is_empty() {
            info!("No github username or password found. Writing sample ron file. Please fill out username and password and try again.");
            match session.save_session() {
                Ok(()) => process::exit(12),
                Err(e) => {
                    error!("{:?}", e);
                    process::exit(24)
                }
            }
        } else {
            session.set_session(
                client
                    .log_in(session.get_username(), session.get_password(), None, None)
                    .await?,
            );
            match session.save_session() {
                Ok(()) => (),
                Err(e) => {
                    error!("{:?}", e);
                    process::exit(24)
                }
            };
        }
    }
    info!(
        "Successfully logged in as {} on {}",
        session.get_username(),
        homeserver_url
    );

    loop {
        let response = match client
            .request(sync_events::Request {
                filter: None,
                since: session.get_last_sync(),
                full_state: false,
                set_presence: SetPresence::Unavailable,
                timeout: Some(Duration::new(2000, 0)),
            })
            .await
        {
            Ok(v) => Some(v),
            Err(e) => {
                debug!("Line 72: {:?}", e);
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
                                            &t,
                                            &m.sender,
                                            room_id,
                                            &client,
                                            session,
                                            searchable_repos,
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

                        session.set_last_sync(Some(v.next_batch.clone()));
                        match session.save_session() {
                            Ok(()) => (),
                            Err(e) => {
                                error!("{:?}", e);
                                process::exit(24)
                            }
                        };
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
                                    if authorized_users.get_authorized_users().contains(&s.sender) {
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
