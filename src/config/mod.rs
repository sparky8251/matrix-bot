use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::fs;
use std::time::SystemTime;

use ruma_client::{
    identifiers::{RoomId, UserId},
    Session,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct BotConfig {
    mx_uname: UserId,
    mx_pass: String,
    gh_uname: String,
    gh_pass: String,
    admins: HashSet<UserId>,
    repos: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RawBotConfig {
    general: RawGeneral,
    matrix_authentication: RawMatrixAuthentication,
    github_authentication: Option<RawGithubAuthentication>,
    searchable_repos: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RawGeneral {
    authorized_users: Option<HashSet<UserId>>,
    enable_corrections: bool,
    enable_unit_conversions: bool,
    insensitive_corrections: Option<Vec<String>>,
    sensitive_corrections: Option<Vec<String>>,
    correction_text: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RawMatrixAuthentication {
    username: Option<UserId>,
    password: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RawGithubAuthentication {
    username: Option<String>,
    password: Option<String>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct SessionStorage {
    session: Option<Session>,
    last_sync: Option<String>,
    last_txn_id: u64,
    last_correction_time: HashMap<RoomId, SystemTime>,
}

pub fn demo_toml() {
    let mut authorized_users = HashSet::new();
    authorized_users.insert(UserId::try_from("@demouser1:matrix.homeserver.com").unwrap());
    authorized_users.insert(UserId::try_from("@demouser2:matrix.homeserver.com").unwrap());
    let mut searchable_repos = HashMap::new();
    searchable_repos.insert("jf".to_string(), "jellyfin/jellyfin".to_string());
    searchable_repos.insert("jf-web".to_string(), "jellyfin/jellyfin-web".to_string());
    let conf = RawBotConfig {
        general: RawGeneral {
            authorized_users: Some(authorized_users),
            enable_corrections: true,
            enable_unit_conversions: true,
            insensitive_corrections: Some(vec!["Jellyfish".to_string(),"Jelly Fin".to_string()]),
            sensitive_corrections: Some(vec!["JellyFin".to_string(),"jellyFin".to_string()]),
            correction_text: Some("I'd just like to interject for a moment {}. What you're referring to as {}, is in fact, Jellyfin, or as I've recently taken to calling it, Emby plus Jellyfin. Jellyfin is not a media server unto itself, but a free component of a media server as defined by Luke Pulverenti. Through a peculiar turn of events, the version of Jellyfin which is widely used today is basically developed with slave labor. Please recognize the harm caused to the slaves by misnaming the project.".to_string())
        },
        matrix_authentication: RawMatrixAuthentication {
            username: Some(UserId::try_from("@botuser:matrix.homeserver.com").unwrap()),
            password: Some("supersecretpassword".to_string()),
        },
        github_authentication: Some(RawGithubAuthentication {
            username: Some("demouser@homeserver.com".to_string()),
            password: Some("supersecretpassword".to_string()),
        }),
        searchable_repos: Some(searchable_repos)
    };

    let toml = toml::to_string_pretty(&conf).unwrap();

    fs::write("test_config.toml", toml).unwrap();
}
