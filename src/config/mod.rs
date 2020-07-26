//! Structs and functions for loading and saving configuration and storage data.

use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::env;
use std::fmt::{Display, Formatter};
use std::fs::{File, OpenOptions};
use std::io::{ErrorKind, Read, Write};
use std::path::PathBuf;
use std::process;
use std::time::{Duration, SystemTime};

use reqwest::header::HeaderValue;
use ruma_client::{
    identifiers::{RoomId, UserId},
    Session,
};
use serde::{Deserialize, Serialize};
use slog::{error, info, trace, Logger};
use url::Url;

/// Constant representing the crate name.
pub const NAME: &str = env!("CARGO_PKG_NAME");
/// Constant representing the crate version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug)]
/// Configuration struct used at runtime. Loaded from RawConfig and its constituent parts.
///
/// Does not have Option<T> fields for ease of use. If its blank it will be a default value or empty.
pub struct Config {
    /// Matrix bot account homeserver URL.
    pub mx_url: Url,
    /// Matrix bot account username.
    pub mx_uname: UserId,
    /// Matrix bot account password.
    pub mx_pass: String,
    /// Github access token as string.
    pub gh_access_token: String,
    /// Bool used to determine if unit conversions will be supported from plain text messages.
    pub enable_unit_conversions: bool,
    /// Bool used to determine if the corrections feature is enabled or not.
    pub enable_corrections: bool,
    /// List of units to exclude from conversions if there is a space between the quantity and unit.
    pub unit_conversion_exclusion: HashSet<String>,
    /// List of all incorrect spellings to match against
    pub incorrect_spellings: Vec<SpellCheckKind>,
    /// Text used in spellcheck correction feature.
    pub correction_text: String,
    /// List of all rooms to be excluded from spellcheck correction feature.
    pub correction_exclusion: HashSet<RoomId>,
    /// List of all words that can be used to link URLs.
    pub linkers: HashSet<String>,
    /// List of matrix users that can invite the bot to rooms.
    pub admins: HashSet<UserId>,
    pub help_rooms: HashSet<RoomId>,
    /// Hashmap containing short name for a repo as a key and the org/repo as a value.
    pub repos: HashMap<String, String>,
    /// Hashmap containing searched key and matching URL for linking.
    pub links: HashMap<String, Url>,
    /// UserAgent used by reqwest
    pub user_agent: HeaderValue,
    /// Hashmap containing group ping name as key and list of user IDs as the value.
    pub group_pings: HashMap<String, HashSet<UserId>>,
    /// Hashset containing list of users that can initiate group pings
    pub group_ping_users: HashSet<UserId>,
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
    linkable_urls: Option<HashMap<String, Url>>,
    /// Hashmap containing group ping name as key and list of user IDs as the value.
    group_pings: Option<HashMap<String, Vec<String>>>,
}

#[derive(Debug, Deserialize)]
/// Struct that contains raw general configuration data.
struct RawGeneral {
    /// List of matrix users that can invite the bot to rooms.
    authorized_users: Option<HashSet<UserId>>,
    help_rooms: Option<HashSet<RoomId>>,
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
    correction_exclusion: Option<HashSet<RoomId>>,
    /// List of all words that can be used to link URLs.
    link_matchers: Option<HashSet<String>>,
}

#[derive(Debug, Deserialize)]
/// Struct that contains raw matrix authentication config data.
struct RawMatrixAuthentication {
    /// Homeserver URL for bot account.
    url: Url,
    /// Matrix username for bot account.
    username: UserId,
    /// Matrix password for bot account.
    password: String,
}

#[derive(Debug, Deserialize)]
/// Struct that contains raw github authentication config data.
struct RawGithubAuthentication {
    /// Access token as string.
    access_token: String,
}

#[derive(Debug, Default, Deserialize, Serialize)]
/// Struct that contains all persistent data the bot modifies during runtime
pub struct Storage {
    /// Last sync token.
    pub last_sync: Option<String>,
    /// Transaction id for last sent message.
    pub last_txn_id: u64,
    /// Matrix session data.
    pub session: Option<Session>,
    /// Hashmap that contains a room id key and a system time of the last correction.
    pub last_correction_time: HashMap<RoomId, SystemTime>,
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
    pub fn load_bot_config(logger: &Logger) -> Self {
        let path = match env::var("MATRIX_BOT_CONFIG_DIR") {
            Ok(v) => [&v, "config.toml"].iter().collect::<PathBuf>(),
            Err(_) => ["config.toml"].iter().collect::<PathBuf>(),
        };
        // File Load Section
        let mut file = match File::open(path) {
            Ok(v) => v,
            Err(e) => match e.kind() {
                ErrorKind::NotFound => {
                    error!(logger, "Unable to find file config.toml");
                    process::exit(1);
                }
                ErrorKind::PermissionDenied => {
                    error!(logger, "Permission denied when opening file config.toml");
                    process::exit(1);
                }
                _ => {
                    error!(
                        logger,
                        "Unable to open file due to unexpected error {:?}", e
                    );
                    process::exit(1);
                }
            },
        };
        let mut contents = String::new();
        match file.read_to_string(&mut contents) {
            Ok(_) => (), // If read is successful, do nothing
            Err(e) => {
                error!(logger, "Unable to read file contents due to error {:?}", e);
                process::exit(2)
            }
        }
        let toml: RawConfig = match toml::from_str(&contents) {
            Ok(v) => v,
            Err(e) => {
                error!(logger, "Invalid toml. Error is {:?}", e);
                process::exit(3)
            }
        };

        // Set variables and exit/error if set improperly
        let (repos, gh_access_token) = load_github_settings(&toml, &logger);
        let (linkers, links) = load_linker_settings(&toml, &logger);
        let unit_conversion_exclusion = load_unit_conversion_settings(&toml, &logger);
        let (incorrect_spellings, correction_text, correction_exclusion) =
            load_spell_correct_settings(&toml, &logger);
        let admins = load_admin_settings(&toml, &logger);
        let help_rooms = load_help_settings(&toml, &logger);
        let (mx_url, mx_uname, mx_pass, enable_corrections, enable_unit_conversions) = (
            toml.matrix_authentication.url.clone(),
            toml.matrix_authentication.username.clone(),
            toml.matrix_authentication.password.clone(),
            toml.general.enable_corrections,
            toml.general.enable_unit_conversions,
        );

        let user_agent: HeaderValue =
            match HeaderValue::from_str(&(NAME.to_string() + "/" + VERSION)) {
                Ok(v) => v,
                Err(e) => panic!(
                    "Unable to create valid user agent from {} and {}. Error is {:?}",
                    NAME, VERSION, e
                ),
            };

        let (group_pings, group_ping_users) = load_group_ping_settings(&toml, &logger);

        // Return value
        Config {
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
            admins,
            help_rooms,
            repos,
            links,
            user_agent,
            group_pings,
            group_ping_users,
        }
    }
}

impl Storage {
    /// Load of bot storage. Used only for startup.
    ///
    /// If the file doesnt exist, creates and writes a default storage file.
    ///
    /// If file exists, attempts load and will exit program if it fails.
    pub fn load_storage(logger: &Logger) -> Self {
        let path = match env::var("MATRIX_BOT_DATA_DIR") {
            Ok(v) => [v, "storage.ron".to_string()].iter().collect::<PathBuf>(),
            Err(_) => ["storage.ron"].iter().collect::<PathBuf>(),
        };
        let mut file = match File::open(path) {
            Ok(v) => v,
            Err(e) => match e.kind() {
                ErrorKind::NotFound => {
                    let ron = Self::default();
                    trace!(logger, "The next save is a default save");
                    Self::save_storage(&ron, &logger);
                    return ron;
                }
                ErrorKind::PermissionDenied => {
                    error!(logger, "Permission denied when opening file storage.ron");
                    process::exit(1);
                }
                _ => {
                    error!(logger, "Unable to open file: {}", e);
                    process::exit(1);
                }
            },
        };
        let mut contents = String::new();
        match file.read_to_string(&mut contents) {
            Ok(_) => (), // If read is successful, do nothing
            Err(e) => {
                error!(logger, "Unable to read file contents: {}", e);
                process::exit(2)
            }
        }
        let ron: Self = match ron::from_str(&contents) {
            Ok(v) => v,
            Err(e) => {
                error!(
                    logger,
                    "Unable to load storage.ron due to invalid ron: {}", e
                );
                process::exit(3)
            }
        };
        ron
    }

    /// Saves all bot associated storage data.
    ///
    /// One of the few functions that can terminate the program if it doesnt go well.
    pub fn save_storage(&self, logger: &Logger) {
        let path = match env::var("MATRIX_BOT_DATA_DIR") {
            Ok(v) => [v, "storage.ron".to_string()].iter().collect::<PathBuf>(),
            Err(_) => ["storage.ron"].iter().collect::<PathBuf>(),
        };
        let ron = match ron::to_string(self) {
            Ok(v) => v,
            Err(e) => {
                error!(
                    logger,
                    "Unable to format storage as ron, this should never occur. Error is {}", e
                );
                process::exit(7)
            }
        };
        let mut file = match OpenOptions::new().write(true).create(true).open(path) {
            Ok(v) => v,
            Err(e) => {
                error!(logger, "Unable to open storage.ron due to error {:?}", e);
                process::exit(9)
            }
        };
        match file.write_all(ron.as_bytes()) {
            Ok(_) => {
                trace!(logger, "Saved Session!");
            }
            Err(e) => {
                error!(logger, "Unable to write storage data: {}", e);
                process::exit(10)
            }
        }
    }

    // FIXME: This needs to be an idempotent/unique ID per txn to be spec compliant
    /// Sets the last_txn_id to a new value then returns it
    ///
    /// Must be saved after it is used successfully or you can cause homeserver issues
    pub fn next_txn_id(&mut self) -> String {
        self.last_txn_id += 1;
        self.last_txn_id.to_string()
    }

    /// Checks that the correction time cooldown for a specific room has passed.
    ///
    /// Returns true if there has never been a correction done in the room before.
    pub fn correction_time_cooldown(&self, room_id: &RoomId) -> bool {
        match self.last_correction_time.get(room_id) {
            Some(t) => match t.elapsed() {
                Ok(d) => d >= Duration::new(300, 0),
                Err(_) => false,
            },
            None => true, // Will only be None if this client has not yet corrected anyone in specified room, so return true to allow correction
        }
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

fn load_github_settings(toml: &RawConfig, logger: &Logger) -> (HashMap<String, String>, String) {
    match &toml.searchable_repos {
        Some(r) => match &toml.github_authentication {
            Some(g) => (r.clone(), g.access_token.clone()),
            None => {
                error!(logger, "Searchable repos configured, but no github access token found. Unable to continue...");
                process::exit(4)
            }
        },
        None => {
            info!(logger, "No searchable repos found. Disabling feature...");
            (HashMap::new(), String::new())
        }
    }
}

fn load_linker_settings(
    toml: &RawConfig,
    logger: &Logger,
) -> (HashSet<String>, HashMap<String, Url>) {
    match &toml.linkable_urls {
        Some(d) => match &toml.general.link_matchers {
            Some(m) => {
                if !d.is_empty() {
                    (m.clone(), d.clone())
                } else {
                    error!(logger, "Link matchers exists but none are set. Exiting...");
                    process::exit(1)
                }
            }
            None => {
                info!(logger, "No link matchers found. Disabling feature...");
                (HashSet::new(), HashMap::new())
            }
        },
        None => {
            info!(logger, "No linkable urls found. Disabling feature...");
            (HashSet::new(), HashMap::new())
        }
    }
}

fn load_unit_conversion_settings(toml: &RawConfig, logger: &Logger) -> HashSet<String> {
    match &toml.general.unit_conversion_exclusion {
        Some(v) => {
            let mut hash_set = HashSet::new();
            for set in v {
                hash_set.insert(" ".to_owned() + &set);
            }
            hash_set
        }
        None => {
            info!(
                logger,
                "No unit conversion exlclusions found. Disabling feature..."
            );
            HashSet::new()
        }
    }
}

fn load_spell_correct_settings(
    toml: &RawConfig,
    logger: &Logger,
) -> (Vec<SpellCheckKind>, String, HashSet<RoomId>) {
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
                                    logger,
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
                            (spk, c.to_string(), e)
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
                            info!(
                                logger,
                                "No list found. No rooms will be excluded from corrections"
                            );
                            (spk, c.to_string(), HashSet::new())
                        }
                    },
                    None => {
                        error!(
                            logger,
                            "No correction text provided even though corrections have been enabled"
                        );
                        process::exit(5)
                    }
                },
                None => {
                    error!(logger, "No case sensitive corrections provided even though corrections have been enabled");
                    process::exit(5)
                }
            },
            None => {
                error!(logger, "No case insensitive corrections provided even though corrections have been enabled");
                process::exit(5)
            }
        }
    } else {
        info!(logger, "Disabling corrections feature");
        (Vec::new(), String::new(), HashSet::new())
    }
}

fn load_admin_settings(toml: &RawConfig, logger: &Logger) -> HashSet<UserId> {
    match &toml.general.authorized_users {
        Some(v) => v.clone(),
        None => {
            error!(logger, "You must provide at least 1 authorized user");
            process::exit(6)
        }
    }
}

fn load_help_settings(toml: &RawConfig, logger: &Logger) -> HashSet<RoomId> {
    match &toml.general.help_rooms {
        Some(v) => v.clone(),
        None => {
            info!(logger, "No help rooms specified. Allowing all rooms.");
            HashSet::new()
        }
    }
}

fn load_group_ping_settings(
    toml: &RawConfig,
    logger: &Logger,
) -> (HashMap<String, HashSet<UserId>>, HashSet<UserId>) {
    match &toml.group_pings {
        Some(v) => {
            let mut group_ping_users = HashSet::new();
            let groups = v.clone();
            for group in groups {
                for user in group.1 {
                    if user.starts_with('@') {
                        let user_id = UserId::try_from(user.clone()).expect(
                            "Somehow got an alias in a part of code meant to handle UserIds",
                        );
                        group_ping_users.insert(user_id);
                    }
                }
            }

            let mut expanded_groups: HashMap<String, HashSet<UserId>> = HashMap::new();
            for (group, users) in v {
                let mut expanded_users: HashSet<UserId> = HashSet::new();

                for user in users {
                    if user.starts_with('%') {
                        // If user is an alias, expand it to list of users and insert them
                        let alias = user.replace("%", "");
                        match v.get(&alias) {
                            // If list of users found, insert them
                            Some(g) => {
                                for u in g {
                                    if u.starts_with('@') {
                                        let user_id = UserId::try_from(u.clone()).expect("Somehow got an alias in a part of code meant to handle UserIds");
                                        expanded_users.insert(user_id);
                                    }
                                }
                            }
                            // If list of users are not found, print error to console and move on
                            None => error!(
                                logger,
                                "Group alias %{} has no corresponding group. Ignoring...", alias
                            ),
                        }
                    } else {
                        // If user is not alias, just insert it
                        let user_id = UserId::try_from(user.clone()).expect(
                            "Somehow got an alias in a part of code meant to handle UserIds",
                        );
                        expanded_users.insert(user_id);
                    }
                }

                expanded_groups.insert(group.to_string(), expanded_users);
            }

            (expanded_groups, group_ping_users)
        }
        None => {
            info!(logger, "No group pings defined. Disabling feature...");
            (HashMap::new(), HashSet::new())
        }
    }
}
