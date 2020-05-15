mod bot;
mod config;
mod handlers;
mod queries;
mod regex;

use bot::Bot;
use config::{Config, Storage};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let storage = Storage::load_storage();
    let config = Config::load_bot_config();
    let api_client = reqwest::Client::new();
    let mut bot = Bot::new(storage, config, api_client);

    {
        let bot_fut = bot.start();
        futures::pin_mut!(bot_fut);

        let ctrlc_fut = tokio::signal::ctrl_c();
        futures::pin_mut!(ctrlc_fut);

        futures::future::select(bot_fut, ctrlc_fut).await;
    }

    bot.storage.save_storage();
}
