//! Structs and functions for loading and saving configuration and storage data.

// TODO: Implement Option type enum that will encapsulate the logic and potential states of config options that
// TODO: are disable-able. This would be to prevent improper use down the line, whereas right now I pass around
// TODO: empty but usable types such as HashMap, which could accidentally be used if I fail to uphold the invariants manually.
// TODO: This problem has gotten worse recently, as now not all empty items mean disabled
// TODO: and as such, the type system needs to come to the rescue

use anyhow::{anyhow, Context};
use axum::http::Uri;
use reqwest::header::HeaderValue;
use ruma::{OwnedRoomId, OwnedUserId, UserId};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::env;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use tracing::info;

/// Constant representing the crate name.
pub const NAME: &str = env!("CARGO_PKG_NAME");
/// Constant representing the crate version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug)]
/// Configuration struct used at runtime. Loaded from RawConfig and its constituent parts.
///
/// Does not have Option<T> fields for ease of use. If its blank it will be a default value or empty.
pub struct MatrixListenerConfig {
    /// Matrix bot account homeserver URL.
    pub mx_url: Uri,
    /// Matrix bot account username.
    pub mx_uname: OwnedUserId,
    /// Matrix bot account password.
    pub mx_pass: Box<str>,
    /// Github access token as string.
    pub gh_access_token: Box<str>,
    /// Bool used to determine if unit conversions will be supported from plain text messages.
    pub enable_unit_conversions: bool,
    /// Bool used to determine if the corrections feature is enabled or not.
    pub enable_corrections: bool,
    /// List of units to exclude from conversions if there is a space between the quantity and unit.
    pub unit_conversion_exclusion: HashSet<Box<str>>,
    /// List of all incorrect spellings to match against
    pub incorrect_spellings: Vec<SpellCheckKind>,
    /// Text used in spellcheck correction feature.
    pub correction_text: Box<str>,
    /// List of all rooms to be excluded from spellcheck correction feature.
    pub correction_exclusion: HashSet<OwnedRoomId>,
    /// List of all words that can be used to link URLs.
    pub linkers: HashSet<Box<str>>,
    /// List of matrix users that can invite the bot to rooms.
    pub admins: HashSet<OwnedUserId>,
    /// List of rooms in which help function can be used.
    pub help_rooms: HashSet<OwnedRoomId>,
    /// List of rooms in which ban function will apply.
    pub ban_rooms: HashSet<OwnedRoomId>,
    /// Hashmap containing short name for a repo as a key and the org/repo as a value.
    pub repos: HashMap<Box<str>, Box<str>>,
    /// Hashmap containing searched key and matching URL for linking.
    pub links: HashMap<Box<str>, Uri>,
    /// List of all text expansions.
    pub text_expansions: HashMap<Box<str>, Box<str>>,
    /// UserAgent used by reqwest
    pub user_agent: HeaderValue,
    /// Hashmap containing group ping name as key and list of user IDs as the value.
    pub group_pings: HashMap<Box<str>, HashSet<OwnedUserId>>,
    /// Hashset containing list of users that can initiate group pings
    pub group_ping_users: HashSet<OwnedUserId>,
}

pub struct WebhookListenerConfig {
    pub token: Box<str>,
}

#[derive(Debug)]
/// Configuration struct used at runtime. Loaded from RawConfig and its constituent parts.
///
/// Does not have Option<T> fields for ease of use. If its blank it will be a default value or empty.
pub struct Config {
    /// Matrix bot account homeserver URL.
    pub mx_url: Uri,
    /// Matrix bot account username.
    pub mx_uname: OwnedUserId,
    /// Matrix bot account password.
    pub mx_pass: Box<str>,
    /// Github access token as string.
    gh_access_token: Box<str>,
    /// Bool used to determine if unit conversions will be supported from plain text messages.
    enable_unit_conversions: bool,
    /// Bool used to determine if the corrections feature is enabled or not.
    enable_corrections: bool,
    /// List of units to exclude from conversions if there is a space between the quantity and unit.
    unit_conversion_exclusion: HashSet<Box<str>>,
    /// List of all incorrect spellings to match against
    incorrect_spellings: Vec<SpellCheckKind>,
    /// Text used in spellcheck correction feature.
    correction_text: Box<str>,
    /// List of all rooms to be excluded from spellcheck correction feature.
    correction_exclusion: HashSet<OwnedRoomId>,
    /// List of all words that can be used to link URLs.
    linkers: HashSet<Box<str>>,
    /// List of matrix users that can invite the bot to rooms.
    admins: HashSet<OwnedUserId>,
    /// List of matrix rooms that the help function can be used in
    help_rooms: HashSet<OwnedRoomId>,
    /// List of matrix rooms in which bans will be applied
    ban_rooms: HashSet<OwnedRoomId>,
    /// Hashmap containing short name for a repo as a key and the org/repo as a value.
    repos: HashMap<Box<str>, Box<str>>,
    /// Hashmap containing searched key and matching URL for linking.
    links: HashMap<Box<str>, Uri>,
    /// List of all text expansions.
    text_expansions: HashMap<Box<str>, Box<str>>,
    /// UserAgent used by reqwest
    user_agent: HeaderValue,
    /// Hashmap containing group ping name as key and list of user IDs as the value.
    group_pings: HashMap<Box<str>, HashSet<OwnedUserId>>,
    /// Hashset containing list of users that can initiate group pings
    group_ping_users: HashSet<OwnedUserId>,
    pub webhook_token: Box<str>,
}

#[derive(Debug, Deserialize)]
/// Struct that represents on disk configuration data.
///
/// Loaded into Config struct at runtime for ease of use.
pub struct RawConfig {
    /// Contains struct for all general configuration data.
    general: RawGeneral,
    /// Contains struct for all matrix authentication data.
    matrix_authentication: RawMatrixAuthentication,
    /// Contains struct for all github authentication data.
    github_authentication: Option<RawGithubAuthentication>,
    /// Hashmap containing short name for a repo as a key and the org/repo as a value.
    searchable_repos: Option<HashMap<String, String>>,
    /// Hashmap containing searched key and matching URL for linking.
    linkable_urls: Option<HashMap<String, String>>,
    /// List of all text expansions.
    text_expansion: Option<HashMap<String, String>>,
    /// Hashmap containing group ping name as key and list of user IDs as the value.
    group_pings: Option<HashMap<String, Vec<String>>>,
}

#[derive(Debug, Deserialize)]
/// Struct that contains raw general configuration data.
struct RawGeneral {
    /// List of matrix users that can invite the bot to rooms.
    authorized_users: Option<HashSet<OwnedUserId>>,
    /// List of rooms the help function can be used in.
    help_rooms: Option<HashSet<OwnedRoomId>>,
    /// List of rooms the ban function will apply to
    ban_rooms: Option<HashSet<OwnedRoomId>>,
    /// Bool used to determine if unit conversions will be supported from plain text messages.
    enable_unit_conversions: bool,
    /// Bool used to determine if the corrections feature is enabled or not.
    enable_corrections: bool,
    /// List of units to exclude from conversions if there is a space between the quantity and unit.
    unit_conversion_exclusion: Option<HashSet<String>>,
    /// List of text that will be matched case insensitively for corrections feature.
    insensitive_corrections: Option<Vec<String>>,
    /// List of text that will be matched case sensitively for corrections feature.
    sensitive_corrections: Option<Vec<String>>,
    /// Text used in spellcheck correction feature. Requires two '{}' to operate properly.
    correction_text: Option<String>,
    /// List of all rooms to be excluded from spellcheck correction feature.
    correction_exclusion: Option<HashSet<OwnedRoomId>>,
    /// List of all words that can be used to link URLs.
    link_matchers: Option<HashSet<String>>,

    webhook_token: String,
}

#[derive(Debug, Deserialize)]
/// Struct that contains raw matrix authentication config data.
struct RawMatrixAuthentication {
    /// Homeserver URL for bot account.
    url: String,
    /// Matrix username for bot account.
    username: OwnedUserId,
    /// Matrix password for bot account.
    password: String,
}

#[derive(Debug, Deserialize)]
/// Struct that contains raw github authentication config data.
struct RawGithubAuthentication {
    /// Access token as string.
    access_token: String,
}

#[derive(Clone, Debug)]
/// Enum you match on to determine if you are doing a case sensitive or insensitive checking
pub enum SpellCheckKind {
    /// Variant that contains a case insesitive string
    SpellCheckInsensitive(InsensitiveSpelling),
    /// Variant that contains a case sensitive string
    SpellCheckSensitive(SensitiveSpelling),
}

#[derive(Clone, Debug)]
/// A struct representing a case insensitive string for comparion purposes.
pub struct InsensitiveSpelling {
    /// The case insensitive string.
    spelling: String,
}

#[derive(Clone, Debug)]
/// A struct representing a case sensitive string for comparison purposes.
pub struct SensitiveSpelling {
    /// The case sensitive string.
    spelling: String,
}

impl MatrixListenerConfig {
    pub fn new(config: &Config) -> Self {
        Self {
            mx_url: config.mx_url.clone(),
            mx_uname: config.mx_uname.clone(),
            mx_pass: config.mx_pass.clone(),
            gh_access_token: config.gh_access_token.clone(),
            enable_unit_conversions: config.enable_unit_conversions,
            enable_corrections: config.enable_corrections,
            unit_conversion_exclusion: config.unit_conversion_exclusion.clone(),
            incorrect_spellings: config.incorrect_spellings.clone(),
            correction_text: config.correction_text.clone(),
            correction_exclusion: config.correction_exclusion.clone(),
            linkers: config.linkers.clone(),
            admins: config.admins.clone(),
            help_rooms: config.help_rooms.clone(),
            ban_rooms: config.ban_rooms.clone(),
            repos: config.repos.clone(),
            links: config.links.clone(),
            text_expansions: config.text_expansions.clone(),
            user_agent: config.user_agent.clone(),
            group_pings: config.group_pings.clone(),
            group_ping_users: config.group_ping_users.clone(),
        }
    }
}

impl Config {
    /// Loads bot config from config.toml.
    ///
    /// Exits program if loading fails.
    ///
    /// Due to the desired structure of the config.toml, this function loads configuration from
    /// a number intermediate structs into the final config struct type used by the program.
    ///
    /// If something is disabled, the value in the final struct is just "new" or "blank" but
    /// does not utilize Option<T> for ease of use and matching later on in the program.
    pub fn load_config() -> anyhow::Result<Self> {
        let path = match env::var("MATRIX_BOT_CONFIG_DIR") {
            Ok(v) => [&v, "config.toml"].iter().collect::<PathBuf>(),
            Err(_) => ["config.toml"].iter().collect::<PathBuf>(),
        };
        // File Load Section
        let mut file = File::open(&path)
            .with_context(|| format!("Unable to open config file at {:?}", path))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .with_context(|| format!("Unable to read file contents at {:?}", path))?;
        let toml: RawConfig = toml::from_str(&contents).context("Invalid toml")?;

        // Set variables and exit/error if set improperly
        let (repos, gh_access_token) = load_github_settings(&toml)?;
        let (linkers, links) = load_linker_settings(&toml)?;
        let text_expansions = load_text_expansions(&toml);
        let unit_conversion_exclusion = load_unit_conversion_settings(&toml);
        let (incorrect_spellings, correction_text, correction_exclusion) =
            load_spell_correct_settings(&toml)?;
        let admins = load_admin_settings(&toml)?;
        let help_rooms = load_help_settings(&toml);
        let ban_rooms = load_ban_room_settings(&toml);
        let (mx_url, mx_uname, mx_pass, enable_corrections, enable_unit_conversions) = (
            toml.matrix_authentication
                .url
                .parse()
                .context("Invalid homeserevr URL")?,
            toml.matrix_authentication.username.clone(),
            toml.matrix_authentication
                .password
                .to_owned()
                .into_boxed_str(),
            toml.general.enable_corrections,
            toml.general.enable_unit_conversions,
        );

        let user_agent =
            HeaderValue::from_str(&(NAME.to_string() + "/" + VERSION)).with_context(|| {
                format!(
                    "Unable to create valid user agent from {} and {}",
                    NAME, VERSION
                )
            })?;

        let (group_pings, group_ping_users) = load_group_ping_settings(&toml)?;
        let webhook_token = toml.general.webhook_token.into_boxed_str();

        // Return value
        Ok(Config {
            mx_url,
            mx_uname,
            mx_pass,
            gh_access_token,
            enable_unit_conversions,
            enable_corrections,
            unit_conversion_exclusion,
            incorrect_spellings,
            correction_text,
            correction_exclusion,
            linkers,
            text_expansions,
            admins,
            help_rooms,
            ban_rooms,
            repos,
            links,
            user_agent,
            group_pings,
            group_ping_users,
            webhook_token,
        })
    }
}

impl From<&str> for InsensitiveSpelling {
    fn from(str: &str) -> Self {
        InsensitiveSpelling {
            spelling: str.to_string(),
        }
    }
}

impl From<&str> for SensitiveSpelling {
    fn from(str: &str) -> Self {
        SensitiveSpelling {
            spelling: str.to_string(),
        }
    }
}

impl Display for SpellCheckKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SpellCheckKind::SpellCheckInsensitive(v) => write!(f, "{}", v),
            SpellCheckKind::SpellCheckSensitive(v) => write!(f, "{}", v),
        }
    }
}

impl Display for InsensitiveSpelling {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.spelling)
    }
}

impl Display for SensitiveSpelling {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.spelling)
    }
}

fn load_github_settings(
    toml: &RawConfig,
) -> anyhow::Result<(HashMap<Box<str>, Box<str>>, Box<str>)> {
    match &toml.searchable_repos {
        Some(r) => match &toml.github_authentication {
            Some(g) => {
                let r = r.iter().map(|(k,v)| (k.clone().into_boxed_str(), v.clone().into_boxed_str())).collect();
                Ok((r, g.access_token.to_owned().into_boxed_str()))
            },
            None => {
                Err(anyhow!(format!("Searchable repos configured, but no github access token found. Unable to continue...")))
            }
        },
        None => {
            info!("No searchable repos found. Disabling feature...");
            Ok((HashMap::new(), String::new().into_boxed_str()))
        }
    }
}

fn load_linker_settings(
    toml: &RawConfig,
) -> anyhow::Result<(HashSet<Box<str>>, HashMap<Box<str>, Uri>)> {
    match &toml.linkable_urls {
        Some(d) => match &toml.general.link_matchers {
            Some(m) => {
                if !d.is_empty() {
                    let d = d
                        .iter()
                        .map(|(k, v)| {
                            (
                                k.clone().into_boxed_str(),
                                v.parse().expect("Invalid URL in linker settings"),
                            )
                        })
                        .collect();
                    let m: HashSet<Box<str>> =
                        m.iter().map(|v| v.clone().into_boxed_str()).collect();
                    Ok((m, d))
                } else {
                    Err(anyhow!(format!(
                        "Link matchers exist, but none are set. Exiting..."
                    )))
                }
            }
            None => {
                info!("No link matchers found. Disabling feature...");
                Ok((HashSet::new(), HashMap::new()))
            }
        },
        None => {
            info!("No linkable urls found. Disabling feature...");
            Ok((HashSet::new(), HashMap::new()))
        }
    }
}

fn load_text_expansions(toml: &RawConfig) -> HashMap<Box<str>, Box<str>> {
    match &toml.text_expansion {
        Some(d) => d
            .iter()
            .map(|(k, v)| (k.clone().into_boxed_str(), v.clone().into_boxed_str()))
            .collect(),
        None => {
            info!("No text expansions found. Disabling Feature...");
            HashMap::new()
        }
    }
}

fn load_unit_conversion_settings(toml: &RawConfig) -> HashSet<Box<str>> {
    match &toml.general.unit_conversion_exclusion {
        Some(v) => {
            let mut hash_set = HashSet::new();
            for set in v {
                hash_set.insert((" ".to_owned() + set).into_boxed_str());
            }
            hash_set
        }
        None => {
            info!("No unit conversion exclusions found. Disabling feature...");
            HashSet::new()
        }
    }
}

fn load_spell_correct_settings(
    toml: &RawConfig,
) -> anyhow::Result<(Vec<SpellCheckKind>, Box<str>, HashSet<OwnedRoomId>)> {
    if toml.general.enable_corrections {
        match &toml.general.insensitive_corrections {
            Some(i) => match &toml.general.sensitive_corrections {
                Some(s) => match &toml.general.correction_text {
                    Some(c) => match &toml.general.correction_exclusion {
                        Some(e) => {
                            let e = if !e.is_empty() {
                                e.clone()
                            } else {
                                info!(
                                    "Empty list found. No rooms will be excluded from corrections"
                                );
                                HashSet::new()
                            };
                            let mut spk = Vec::new();
                            for spelling in i {
                                spk.push(SpellCheckKind::SpellCheckInsensitive(
                                    InsensitiveSpelling {
                                        spelling: spelling.clone(),
                                    },
                                ));
                            }
                            for spelling in s {
                                spk.push(SpellCheckKind::SpellCheckSensitive(SensitiveSpelling {
                                    spelling: spelling.clone(),
                                }));
                            }
                            Ok((spk, c.to_string().into_boxed_str(), e))
                        }
                        None => {
                            let mut spk = Vec::new();
                            for spelling in i {
                                spk.push(SpellCheckKind::SpellCheckInsensitive(
                                    InsensitiveSpelling {
                                        spelling: spelling.clone(),
                                    },
                                ));
                            }
                            for spelling in s {
                                spk.push(SpellCheckKind::SpellCheckSensitive(SensitiveSpelling {
                                    spelling: spelling.clone(),
                                }));
                            }
                            info!("No list found. No rooms will be excluded from corrections");
                            Ok((spk, c.to_string().into_boxed_str(), HashSet::new()))
                        }
                    },
                    None => {
                        Err(anyhow!(format!("No correction text provided, even though corrections have been enabled.")))
                    }
                },
                None => {
                    Err(anyhow!(format!("No case sensitive corrections provided even though case corrections have been enabled.")))
                }
            },
            None => {
                Err(anyhow!(format!("No case insensitive corrections provided even though corrections have been enabled.")))
            }
        }
    } else {
        info!("Disabling corrections feature");
        Ok((Vec::new(), String::new().into_boxed_str(), HashSet::new()))
    }
}

fn load_admin_settings(toml: &RawConfig) -> anyhow::Result<HashSet<OwnedUserId>> {
    match &toml.general.authorized_users {
        Some(v) => Ok(v.clone()),
        None => Err(anyhow!(format!(
            "You must provide at least 1 authorized user"
        ))),
    }
}

fn load_help_settings(toml: &RawConfig) -> HashSet<OwnedRoomId> {
    match &toml.general.help_rooms {
        Some(v) => v.clone(),
        None => {
            info!("No help rooms specified. Allowing all rooms.");
            HashSet::new()
        }
    }
}

fn load_ban_room_settings(toml: &RawConfig) -> HashSet<OwnedRoomId> {
    match &toml.general.ban_rooms {
        Some(v) => v.clone(),
        None => {
            info!("No ban rooms specified. Disabling feature.");
            HashSet::new()
        }
    }
}

fn load_group_ping_settings(
    toml: &RawConfig,
) -> anyhow::Result<(
    HashMap<Box<str>, HashSet<OwnedUserId>>,
    HashSet<OwnedUserId>,
)> {
    match &toml.group_pings {
        Some(v) => {
            let mut group_ping_users = HashSet::new();
            let groups = v.clone();
            for group in groups {
                for user in group.1 {
                    if user.eq("%all") {
                        return Err(anyhow!(format!(
                            "%all is a reserved group_ping name, do not configure it manually"
                        )));
                    }
                    if user.starts_with('@') {
                        let user_id = UserId::parse(user.clone()).context(
                            "Somehow got an alias in a part of code meant to handle UserIds",
                        )?;
                        group_ping_users.insert(user_id);
                    }
                }
            }

            let mut expanded_groups: HashMap<Box<str>, HashSet<OwnedUserId>> = HashMap::new();
            for (group, users) in v {
                let mut expanded_users: HashSet<OwnedUserId> = HashSet::new();

                for user in users {
                    if user.eq("%all") {
                        return Err(anyhow!(format!(
                            "%all is a reserved group_ping name, do not configure it manually"
                        )));
                    }
                    if user.starts_with('%') {
                        // If user is an alias, expand it to list of users and insert them
                        let alias = user.replace('%', "");
                        match v.get(&alias) {
                            // If list of users found, insert them
                            Some(g) => {
                                for u in g {
                                    if u.starts_with('@') {
                                        let user_id = UserId::parse(u.clone()).context("Somehow got an alias in a part of code meant to handle UserIds")?;
                                        expanded_users.insert(user_id);
                                    }
                                }
                            }
                            // If list of users are not found, print error to console and move on
                            None => {
                                return Err(anyhow!(format!(
                                    "Group alias %{} has no corresponding group. Ignoring...",
                                    alias
                                )))
                            }
                        }
                    } else {
                        // If user is not alias, just insert it
                        let user_id = UserId::parse(user.clone()).context(
                            "Somehow got an alias in a part of code meant to handle UserIds",
                        )?;
                        expanded_users.insert(user_id);
                    }
                }

                expanded_groups.insert(group.clone().into_boxed_str(), expanded_users);
            }

            Ok((expanded_groups, group_ping_users))
        }
        None => {
            info!("No group pings defined. Disabling feature...");
            Ok((HashMap::new(), HashSet::new()))
        }
    }
}
