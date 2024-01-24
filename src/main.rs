//! # Matrix Bot
//!
//! Matrix Bot is a simple matrix bot aimed at medium to large projects that span many chat rooms and many active repos looking for more options than the official matrix bots can provide.
//!
//! ## Features
//!
//! - ### Accepts invites to join rooms from authorized users
//!
//!     List is configrable but requires at least 1 entry
//!
//! - ### A jokey interjection that can correct pesky users that misspell your project name
//!
//!     Off by default
//!
//!     Can prevent from running in specific rooms
//!
//!     Has fully configurable matches (case sensitive *and* insentive) and response text
//!
//!     Has 5 minute cooldown timer per room for less spam and more fun
//!
//! - ### An imperial <--> metric converter for all messages containing common units
//!
//!     Can be disabled entirely
//!
//!     Configurable exclution of units from spaced matches if they are also a word (eg: 'i got a 500 in response')
//!
//!     Eases idle chitchat between community members
//!
//! - ### A configurable search and link for issues/pulls in any Github repos the supplied Github access token can see
//!     This can be turned off by not supplying any repos to search
//!
//!     Searches are parsed from message text if they match 'jf#123' or 'jf #123'
//!
//!     The left side of the # is configrable and can point to any repo
//!
//!     Uses GraphQL to be API cost effective (REST might require 2 hits depending on returned result)
//!
//! - ### A configrable general purpose linker
//!
//!     This can be turned off by supplying no linkable urls
//!
//!     Links anything matched from a parsed message if it contains 'docs@hwa' or 'link @troubleshooting'
//!
//!     Left side is configurable. All linkable urls can be triggered with all keywords
//!
//!     Right side is configrable and can link to any URL
//!
//! ## Installation and configuration
//!
//! Currently there is no package or release binary produced. To install first clone the repo and make a release build
//!
//! Collect the `target/release/matrix-bot` binary and place it where you want to run it from
//!
//! Copy the `sample_config.toml` to `config.toml` and put it next to the binary
//!
//! Configure the bot as required with the help of the comments
//!
//! `./matrix-bot` to run
//!
//! I hope you enjoy your experience and please report and issues or feature requests you might have

#![forbid(unsafe_code)]
#![warn(missing_docs)]
//#![warn(clippy::missing_docs_in_private_items)]

mod bot;
mod config;
mod database;
mod events;
mod helpers;
mod logging;
mod messages;
mod queries;
mod regex;
mod services;

#[tokio::main]
#[allow(clippy::missing_docs_in_private_items)]
async fn main() -> anyhow::Result<()> {
    logging::init();
    bot::init().await?;
    Ok(())
}
