//! Structs and functions for loading and saving configuration and storage data.

use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::fs::{File, OpenOptions};
use std::io::{ErrorKind, Read, Write};
use std::process;
use std::time::{Duration, SystemTime};

use log::{error, info, trace};
use reqwest::header::HeaderValue;
use ruma_client::{
    identifiers::{RoomId, UserId},
    Session,
};
use serde::{Deserialize, Serialize};
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
    /// Hashmap containing short name for a repo as a key and the org/repo as a value.
    pub repos: HashMap<String, String>,
    /// Hashmap containing searched key and matching URL for linking.
    pub links: HashMap<String, Url>,
    /// UserAgent used by reqwest
    pub user_agent: HeaderValue,
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
}

#[derive(Debug, Deserialize)]
/// Struct that contains raw general configuration data.
struct RawGeneral {
    /// List of matrix users that can invite the bot to rooms.
    authorized_users: Option<HashSet<UserId>>,
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
    #[allow(clippy::cognitive_complexity)]
    /// Loads bot config from config.toml.
    /// 
    /// Exits program if loading fails.
    /// 
    /// Due to the desired structure of the config.toml, this function loads configuration from
    /// a number intermediate structs into the final config struct type used by the program.
    /// 
    /// If something is disabled, the value in the final struct is just "new" or "blank" but
    /// does not utilize Option<T> for ease of use and matching later on in the program.
    pub fn load_bot_config() -> Self {
        // File Load Section
        let mut file = match File::open("config.toml") {
            Ok(v) => v,
            Err(e) => match e.kind() {
                ErrorKind::NotFound => {
                    error!("Unable to find file config.toml");
                    process::exit(1);
                }
                ErrorKind::PermissionDenied => {
                    error!("Permission denied when opening file config.toml");
                    process::exit(1);
                }
                _ => {
                    error!("Unable to open file due to unexpected error {:?}", e);
                    process::exit(1);
                }
            },
        };
        let mut contents = String::new();
        match file.read_to_string(&mut contents) {
            Ok(_) => (), // If read is successful, do nothing
            Err(e) => {
                error!("Unable to read file contents due to error {:?}", e);
                process::exit(2)
            }
        }
        let toml: RawConfig = match toml::from_str(&contents) {
            Ok(v) => v,
            Err(e) => {
                error!("Invalid toml. Error is {:?}", e);
                process::exit(3)
            }
        };

        // Set variables and exit/error if set improperly
        let (repos, gh_access_token) = match toml.searchable_repos {
            Some(r) => match toml.github_authentication {
                Some(g) => (r, g.access_token),
                None => {
                    error!("Searchable repos configured, but no github access token found. Unable to continue...");
                    process::exit(4)
                }
            },
            None => {
                info!("No searchable repos found. Disabling feature...");
                (HashMap::new(), String::new())
            }
        };
        let (linkers, links) = match toml.linkable_urls {
            Some(d) => match toml.general.link_matchers {
                Some(m) => {
                    if !d.is_empty() {
                        (m, d)
                    } else {
                        error!("Link matchers exists but none are set. Exiting...");
                        process::exit(1)
                    }
                }
                None => {
                    info!("No link matchers found. Disabling feature...");
                    (HashSet::new(), HashMap::new())
                }
            },
            None => {
                info!("No linkable urls found. Disabling feature...");
                (HashSet::new(), HashMap::new())
            }
        };
        let unit_conversion_exclusion = match toml.general.unit_conversion_exclusion {
            Some(v) => {
                let mut hash_set = HashSet::new();
                for set in v {
                    hash_set.insert(" ".to_owned() + &set);
                }
                hash_set
            }
            None => {
                info!("No unit conversion exlclusions found. Disabling feature...");
                HashSet::new()
            }
        };
        let (incorrect_spellings, correction_text, correction_exclusion) = if toml
            .general
            .enable_corrections
        {
            match toml.general.insensitive_corrections {
                Some(i) => match toml.general.sensitive_corrections {
                    Some(s) => match toml.general.correction_text {
                        Some(c) => match toml.general.correction_exclusion {
                            Some(e) => {
                                let e = if !e.is_empty() {
                                    e
                                } else {
                                    info!("Empty list found. No rooms will be excluded from corrections");
                                    HashSet::new()
                                };
                                let mut spk = Vec::new();
                                for spelling in i {
                                    spk.push(SpellCheckKind::SpellCheckInsensitive(
                                        InsensitiveSpelling { spelling },
                                    ));
                                }
                                for spelling in s {
                                    spk.push(SpellCheckKind::SpellCheckSensitive(
                                        SensitiveSpelling { spelling },
                                    ));
                                }
                                (spk, c, e)
                            }
                            None => {
                                let mut spk = Vec::new();
                                for spelling in i {
                                    spk.push(SpellCheckKind::SpellCheckInsensitive(
                                        InsensitiveSpelling { spelling },
                                    ));
                                }
                                for spelling in s {
                                    spk.push(SpellCheckKind::SpellCheckSensitive(
                                        SensitiveSpelling { spelling },
                                    ));
                                }
                                info!("No list found. No rooms will be excluded from corrections");
                                (spk, c, HashSet::new())
                            }
                        },
                        None => {
                            error!("No correction text provided even though corrections have been enabled");
                            process::exit(5)
                        }
                    },
                    None => {
                        error!("No case sensitive corrections provided even though corrections have been enabled");
                        process::exit(5)
                    }
                },
                None => {
                    error!("No case insensitive corrections provided even though corrections have been enabled");
                    process::exit(5)
                }
            }
        } else {
            info!("Disabling corrections feature");
            (Vec::new(), String::new(), HashSet::new())
        };
        let admins = match toml.general.authorized_users {
            Some(v) => v,
            None => {
                error!("You must provide at least 1 authorized user");
                process::exit(6)
            }
        };
        let (mx_url, mx_uname, mx_pass, enable_corrections, enable_unit_conversions) = (
            toml.matrix_authentication.url,
            toml.matrix_authentication.username,
            toml.matrix_authentication.password,
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
            repos,
            links,
            user_agent,
        }
    }
}

impl Storage {
    /// Load of bot storage. Used only for startup.
    /// 
    /// If the file doesnt exist, creates and writes a default storage file.
    /// 
    /// If file exists, attempts load and will exit program if it fails.
    pub fn load_storage() -> Self {
        let mut file = match File::open("storage.toml") {
            Ok(v) => v,
            Err(e) => match e.kind() {
                ErrorKind::NotFound => {
                    let toml = Self::default();
                    trace!("The next save is a default save");
                    Self::save_storage(&toml);
                    return toml;
                }
                ErrorKind::PermissionDenied => {
                    error!("Permission denied when opening file storage.toml");
                    process::exit(1);
                }
                _ => {
                    error!("Unable to open file due to unexpected error {:?}", e);
                    process::exit(1);
                }
            },
        };
        let mut contents = String::new();
        match file.read_to_string(&mut contents) {
            Ok(_) => (), // If read is successful, do nothing
            Err(e) => {
                error!("Unable to read file contents due to error {:?}", e);
                process::exit(2)
            }
        }
        let toml: Self = match toml::from_str(&contents) {
            Ok(v) => v,
            Err(e) => {
                error!("Invalid toml. Error is {:?}", e);
                process::exit(3)
            }
        };
        toml
    }

    /// Saves all bot associated storage data.
    /// 
    /// One of the few functions that can terminate the program if it doesnt go well.
    pub fn save_storage(&self) {
        let toml = match toml::to_string_pretty(self) {
            Ok(v) => v,
            Err(e) => {
                error!(
                    "Unable to format storage as toml, this should never occur. Error is {:?}",
                    e
                );
                process::exit(7)
            }
        };
        let mut file = match OpenOptions::new()
            .write(true)
            .create(true)
            .open("storage.toml")
        {
            Ok(v) => v,
            Err(e) => {
                error!("Unable to open storage.toml due to error {:?}", e);
                process::exit(9)
            }
        };
        match file.write_all(toml.as_bytes()) {
            Ok(_) => {
                trace!("Saved Session!");
            }
            Err(e) => {
                error!("Unable to write storage data to disk due to error {:?}", e);
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
