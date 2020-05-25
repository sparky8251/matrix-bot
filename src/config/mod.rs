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

pub const NAME: &'static str = env!("CARGO_PKG_NAME");
pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[derive(Debug)]
pub struct Config {
    pub mx_url: Url,
    pub mx_uname: UserId,
    pub mx_pass: String,
    pub gh_access_token: String,
    pub enable_corrections: bool,
    pub enable_unit_conversions: bool,
    pub incorrect_spellings: Vec<SpellCheckKind>,
    pub correction_text: String,
    pub linkers: HashSet<String>,
    pub admins: HashSet<UserId>,
    pub repos: HashMap<String, String>,
    pub docs: HashMap<String, String>,
    pub user_agent: HeaderValue,
}

#[derive(Debug, Deserialize)]
pub struct RawConfig {
    general: RawGeneral,
    matrix_authentication: RawMatrixAuthentication,
    github_authentication: Option<RawGithubAuthentication>,
    searchable_repos: Option<HashMap<String, String>>,
    linkable_docs: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
struct RawGeneral {
    authorized_users: Option<HashSet<UserId>>,
    enable_corrections: bool,
    enable_unit_conversions: bool,
    insensitive_corrections: Option<Vec<String>>,
    sensitive_corrections: Option<Vec<String>>,
    correction_text: Option<String>,
    link_matchers: Option<HashSet<String>>,
}

#[derive(Debug, Deserialize)]
struct RawMatrixAuthentication {
    url: Url,
    username: UserId,
    password: String,
}

#[derive(Debug, Deserialize)]
struct RawGithubAuthentication {
    access_token: String,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Storage {
    pub last_sync: Option<String>,
    pub last_txn_id: u64,
    pub session: Option<Session>,
    pub last_correction_time: HashMap<RoomId, SystemTime>,
}

#[derive(Clone, Debug)]
pub enum SpellCheckKind {
    SpellCheckInsensitive(InsensitiveSpelling),
    SpellCheckSensitive(SensitiveSpelling),
}
#[derive(Clone, Debug)]
pub struct InsensitiveSpelling {
    spelling: String,
}

#[derive(Clone, Debug)]
pub struct SensitiveSpelling {
    spelling: String,
}

impl Config {
    // TODO: Change return type to Result<_, _>
    // Implement tests with sample configs based on the returned result
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
        let (linkers, docs) = match toml.linkable_docs {
            Some(d) => match toml.general.link_matchers {
                Some(m) => {
                    if d.len() != 0 {
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
                info!("No linkable docs found. Disabling feature...");
                (HashSet::new(), HashMap::new())
            }
        };
        let (incorrect_spellings, correction_text) = match toml.general.enable_corrections {
            true => match toml.general.insensitive_corrections {
                Some(i) => match toml.general.sensitive_corrections {
                    Some(s) => match toml.general.correction_text {
                        Some(c) => {
                            let mut spk = Vec::new();
                            for spelling in i {
                                spk.push(SpellCheckKind::SpellCheckInsensitive(
                                    InsensitiveSpelling { spelling },
                                ));
                            }
                            for spelling in s {
                                spk.push(SpellCheckKind::SpellCheckSensitive(SensitiveSpelling {
                                    spelling,
                                }));
                            }
                            (spk, c)
                        }
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
            },
            false => {
                info!("Disabling corrections feature");
                (Vec::new(), String::new())
            }
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
            enable_corrections,
            enable_unit_conversions,
            incorrect_spellings,
            correction_text,
            linkers,
            admins,
            repos,
            docs,
            user_agent,
        }
    }
}

impl Storage {
    // TODO: Change return type to Result<_, _>
    // Implement tests with sample storage files based on the returned result
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
        return toml;
    }

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
    pub fn next_txn_id(&mut self) -> String {
        self.last_txn_id += 1;
        self.last_txn_id.to_string()
    }

    pub fn correction_time_cooldown(&self, room_id: &RoomId) -> bool {
        match self.last_correction_time.get(room_id) {
            Some(t) => match t.elapsed() {
                Ok(d) => {
                    if d >= Duration::new(300, 0) {
                        true
                    } else {
                        false
                    }
                }
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
