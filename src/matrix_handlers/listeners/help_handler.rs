use crate::config::MatrixListenerConfig;
use crate::helpers::MatrixFormattedNoticeResponse;
use crate::messages::{MatrixFormattedMessage, MatrixMessage, MatrixMessageType};
use ruma::{events::room::message::TextMessageEventContent, RoomId};
use std::convert::From;
use tokio::sync::mpsc::Sender;
use tracing::{debug, error, trace};

#[derive(Debug)]
enum HelpType {
    Command,
    Commandless,
    GroupPing,
    GithubSearch,
    Link,
    TextExpansion,
    UnitConversion,
    UnknownCommand,
}

impl From<&str> for HelpType {
    fn from(value: &str) -> Self {
        match value.to_ascii_lowercase().as_ref() {
            "command" => HelpType::Command,
            "commandless" => HelpType::Commandless,
            "ping" => HelpType::GroupPing,
            "github-search" => HelpType::GithubSearch,
            "link" => HelpType::Link,
            "text-expansion" => HelpType::TextExpansion,
            "unit-conversion" => HelpType::UnitConversion,
            _ => HelpType::UnknownCommand,
        }
    }
}

pub(super) async fn help_handler(
    text: &TextMessageEventContent,
    room_id: &RoomId,
    config: &MatrixListenerConfig,
    send: &mut Sender<MatrixMessage>,
) {
    if config.help_rooms.is_empty() || config.help_rooms.contains(room_id) {
        trace!("Room is allowed, building help message");
        let mut message = String::new();
        match text.body.split(' ').nth(1).map(HelpType::from) {
            Some(v) => match v {
                HelpType::Command => message = action_command_help_message(),
                HelpType::Commandless => message = action_commandless_help_message(),
                HelpType::GroupPing => message = group_ping_help_message(config),
                HelpType::GithubSearch => message = github_search_help_message(config),
                HelpType::Link => message = link_help_message(config),
                HelpType::TextExpansion => message = text_expansion_help_message(config),
                HelpType::UnitConversion => message = unit_conversion_help_message(config),
                HelpType::UnknownCommand => (),
            },
            None => {
                trace!("Printing help message for program");
                message = generic_help_message();
            }
        };
        if !message.is_empty() {
            if send
                .send(MatrixMessage {
                    room_id: Some(room_id.to_owned()),
                    message: MatrixMessageType::Notice(message),
                })
                .await
                .is_err()
            {
                error!("Channel closed. Unable to send message.");
            }
        } else {
            debug!("Unknown action");
            let mut response = MatrixFormattedNoticeResponse::default();
            let mut errors = Vec::new();
            errors.push(format!(
                "Unknown action {}",
                text.body.split(' ').nth(1).unwrap_or("")
            ));
            response.add_errrors(errors);
            let formatted_text = response.format_text();
            if send
                .send(MatrixMessage {
                    room_id: Some(room_id.to_owned()),
                    message: MatrixMessageType::FormattedNotice(MatrixFormattedMessage {
                        plain_text: response.to_string(),
                        formatted_text,
                    }),
                })
                .await
                .is_err()
            {
                error!("Channel closed. Unable to send message.");
            }
        }
    } else {
        trace!(
            "Rooms are limited and room {} is not in the allowed list of help command rooms",
            room_id
        );
    }
}

fn generic_help_message() -> String {
    format!("Matrix Bot v{}
Repository: {}

This bot has two types of actions it can perform: command and commandless
Use the !help command to learn more about their characteristics

USAGE:
\t!help command|commandless
\t!help [ACTION]

ACTION TYPES:
\tcommand\t\tCommand actions are a message that starts with !
\tcommandless\tCommandless actions are any message that meets the critera to trigger an action and do not start with an !

ACTIONS:
\tping\t\t\tPing a group of people
\tgithub-search\tSearch github by project and issue/PR number
\tlink\t\t\t\tShortcuts for linking helpful URLs
\tunit-conversion\tConvert common conversational units",
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_REPOSITORY")
    )
}

fn action_command_help_message() -> String {
    "Command Action

Command actions are defined as message that have no formatting (like no italics, no inline code, not a reply, etc) that start with a !. These can only perform one action per message.

EXAMPLES:
\t!help
\t!convert 22mi".to_string()
}

fn action_commandless_help_message() -> String {
    "Commandless Action

Commandles actions can happen in any plain text message but certain text formatting will be ignored. Currently ignored formatting is inline code, code blocks, and the text in a reply (but not the reply itself)

The exact rules for triggering a commandless action vary by action (so check action help pages for info on how to trigger them), but their defining features are that they can be in any part of a message and mutiple can be triggered per message.

EXAMPLES:
\tHey there, i think you want to read docs@troubleshooting
\tIts not like 32f is that cold. not sure what you are complaining about
".to_string()
}

fn group_ping_help_message(config: &MatrixListenerConfig) -> String {
    let mut groups: Vec<&String> = config.group_pings.keys().collect();
    groups.sort();
    let mut available_groups = String::new();
    for group in &groups[..(groups.len() - 1)] {
        available_groups.push_str(group);
        available_groups.push_str(" | ");
    }

    available_groups.push_str(groups[groups.len() - 1]);

    format!("Group Ping

This action is only available as commandless. It will trigger on anything that matches \"%group\" where \"group\" is the group you want to ping.

If the group exists and you are authorized to make a group ping, a message pinging everyone in the group will be made in a bot message.

USAGE:
\tHey there %server can you look at this for me?
\t%server

AVAILABLE GROUPS:
{}", available_groups
    )
}

fn github_search_help_message(config: &MatrixListenerConfig) -> String {
    let mut repos: Vec<&String> = config.repos.keys().collect();
    repos.sort();
    let mut available_repos = String::new();
    for repo in &repos[..(repos.len() - 1)] {
        available_repos.push_str(repo);
        available_repos.push_str(" | ");
    }

    available_repos.push_str(repos[repos.len() - 1]);

    format!("Github Search

This action is only available as commandless. It will trigger on anything that matches \"jf#1234\" where \"jf\" is the repo you want to search and \"1234\" is the issue or PR you want to link.

If the repo and the number exist, it will provide a link to the issue or pull in a bot message.

USAGE:
\tI could use a review on jf#1234
\tjf#1234

AVAILABLE REPOS:
{}", available_repos)
}

fn link_help_message(config: &MatrixListenerConfig) -> String {
    let mut keywords: Vec<&String> = config.linkers.iter().collect();
    keywords.sort();
    let mut available_keywords = String::new();
    for keyword in &keywords[..(keywords.len() - 1)] {
        available_keywords.push_str(keyword);
        available_keywords.push_str(" | ");
    }

    available_keywords.push_str(keywords[keywords.len() - 1]);

    let mut links: Vec<&String> = config.links.keys().collect();
    links.sort();
    let mut available_links = String::new();
    for link in &links[..(links.len() - 1)] {
        available_links.push_str(link);
        available_links.push_str(" | ");
    }

    available_links.push_str(links[links.len() - 1]);

    format!("Link

This action is only available as commandless. It will trigger on anything that matches \"link@hwa\" where \"link\" is a configured keyword and \"hwa\" is a linkable item.

if the keyword and item exist, there will be a link provided in a bot message.

USAGE:
\tI think you might want to look at link@hwa
\tlink@hwa

AVAILABLE KEYWORDS:
{}

AVAILABLE LINKS:
{}
    ", available_keywords, available_links)
}

fn text_expansion_help_message(config: &MatrixListenerConfig) -> String {
    let mut keywords: Vec<&String> = config.text_expansions.keys().collect();
    keywords.sort();
    let mut available_keywords = String::new();
    for keyword in &keywords[..(keywords.len() - 1)] {
        available_keywords.push_str(keyword);
        available_keywords.push_str(" | ");
    }

    available_keywords.push_str(keywords[keywords.len() - 1]);

    format!("Text Expansion

This action is only available as commandless. It will trigger on anything that matches \"$text\" where \"text\" is a configured keyword.

if the keyword exists, there will be a message containing designated expanded text provided in a bot message.

USAGE:
\tIf you have questions about the addon, i hope $kodi answers it for you
\t$kodi

AVAILABLE KEYWORDS:
{}
    ", available_keywords)
}

fn unit_conversion_help_message(config: &MatrixListenerConfig) -> String {
    let mut units: Vec<&String> = config.unit_conversion_exclusion.iter().collect();
    units.sort();
    let mut space_excluded_units = String::new();
    for unit in &units[..(units.len() - 1)] {
        space_excluded_units.push_str(unit);
        space_excluded_units.push_str(" | ");
    }

    space_excluded_units.push_str(units[units.len() - 1]);

    format!("Unit Conversion

This action is available as both a command and commanless. It will convert common converstation units Imperial <-> Metric to help ease international chat. There can be a space between the quantity and unit except for the units excluded by configuration (listed below).

USAGE:
\tCOMMAND:
\t\t!convert 20c

\tCOMMANDLESS:
\t\tIt's weird that the speed limit here is 45mph
\t\t45 mph

SUPPORTED UNITS:
LENGTH:
cm | m | km | in | ft | mi | mile | miles
TEMPERATURE:
c | °c | f | °f
WEIGHT:
kg | lbs
SPEED:
km/h | kmh | kph | kmph | mph

SPACE EXCLUDED UNITS:
{}
    ", space_excluded_units)
}
