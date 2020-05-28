#![forbid(unsafe_code)]
// #![deny(warnings)]

mod bot;
mod config;
mod handlers;
mod helpers;
mod queries;
mod regex;

use bot::Bot;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let mut bot = Bot::new();

    {
        let bot_fut = bot.start();
        futures::pin_mut!(bot_fut);

        let ctrlc_fut = tokio::signal::ctrl_c();
        futures::pin_mut!(ctrlc_fut);

        futures::future::select(bot_fut, ctrlc_fut).await;
    }

    bot.storage.save_storage();
}
