mod bot;
mod handlers;
mod regex;
mod session;

mod config;

use std::process;

use log::error;
use session::{AuthorizedUsers, SavedSession, SearchableRepos};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let mut session = SavedSession::load_session().unwrap_or_default();
    let authorized_users = AuthorizedUsers::load_authorized_users().unwrap_or_default();
    let searchable_repos = SearchableRepos::load_searchable_repos().unwrap_or_default();
    let api_client = reqwest::Client::new();

    {
        let bot_fut = bot::start(
            "https://matrix.possumlodge.me".parse().unwrap(),
            &mut session,
            &authorized_users,
            &searchable_repos,
            &api_client,
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
