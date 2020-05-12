mod bot;
mod handlers;
mod regex;
mod session;

use std::process;

use log::error;
use session::{AuthorizedUsers, SavedSession};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let mut session = SavedSession::load_session().unwrap_or_default();
    let authorized_users = AuthorizedUsers::load_authorized_users().unwrap_or_default();

    {
        let bot_fut = bot::start(
            "https://matrix.possumlodge.me".parse().unwrap(),
            &mut session,
            &authorized_users,
        );
        futures::pin_mut!(bot_fut);

        let ctrlc_fut = tokio::signal::ctrl_c();
        futures::pin_mut!(ctrlc_fut);

        futures::future::select(bot_fut, ctrlc_fut).await;
    }

    match session.save_session() {
        Ok(()) => (),
        Err(e) => {
            error!("{:?}", e);
            process::exit(24)
        }
    };
}
