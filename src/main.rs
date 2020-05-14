mod bot;
mod config;
mod handlers;
mod regex;

use config::{BotConfig, Storage};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let mut session = Storage::load_storage();
    let config = BotConfig::load_bot_config();
    let api_client = reqwest::Client::new();

    {
        let bot_fut = bot::start(&mut session, &config, &api_client);
        futures::pin_mut!(bot_fut);

        let ctrlc_fut = tokio::signal::ctrl_c();
        futures::pin_mut!(ctrlc_fut);

        futures::future::select(bot_fut, ctrlc_fut).await;
    }

    session.save_storage();
}
