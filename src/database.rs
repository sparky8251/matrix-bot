use anyhow::{anyhow, Context};
use ruma::{OwnedRoomId, RoomId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs::{File, OpenOptions};
use std::io::{ErrorKind, Read, Write};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use tracing::trace;

#[derive(Debug, Default, Deserialize, Serialize)]
/// Struct that contains persistent matrix listener data the bot modifies during runtime
pub struct ListenerStorage {
    /// Last sync token.
    pub last_sync: Option<String>,
    /// Hashmap that contains a room id key and a system time of the last correction.
    pub last_correction_time: HashMap<OwnedRoomId, SystemTime>,
}

impl ListenerStorage {
    /// Load of bot storage. Used only for startup.
    ///
    /// If the file doesnt exist, creates and writes a default storage file.
    ///
    /// If file exists, attempts load and will exit program if it fails.
    pub fn load_storage() -> anyhow::Result<Self> {
        let path = match env::var("MATRIX_BOT_DATA_DIR") {
            Ok(v) => [v, "matrix_listener.ron".to_string()]
                .iter()
                .collect::<PathBuf>(),
            Err(_) => ["matrix_listener.ron"].iter().collect::<PathBuf>(),
        };
        let mut file = match File::open(path) {
            Ok(v) => v,
            Err(e) => match e.kind() {
                ErrorKind::NotFound => {
                    let ron = Self::default();
                    trace!("The next save is a default save");
                    Self::save_storage(&ron)
                        .context("Unable to save default matrix_listener.ron")?;
                    return Ok(ron);
                }
                ErrorKind::PermissionDenied => {
                    return Err(anyhow!(
                        "Permission denied when opening file matrix_listener.ron"
                    ));
                }
                _ => {
                    return Err(anyhow!("Unable to open matrix_listener.ron"));
                }
            },
        };
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .context("Unable to read file contents of matrix_listener.ron")?;
        let ron = ron::from_str(&contents)
            .context("Unable to load matrix_listener.ron due to invald RON")?;
        Ok(ron)
    }

    /// Saves all bot associated storage data.
    ///
    /// One of the few functions that can terminate the program if it doesnt go well.
    pub fn save_storage(&self) -> anyhow::Result<()> {
        let path = match env::var("MATRIX_BOT_DATA_DIR") {
            Ok(v) => [v, "matrix_listener.ron".to_string()]
                .iter()
                .collect::<PathBuf>(),
            Err(_) => ["matrix_listener.ron"].iter().collect::<PathBuf>(),
        };
        let ron = ron::to_string(self).context(
            "Unable to format matrix_listener.ron save data as RON. This should never occur!",
        )?;
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(path)
            .context("Unable to open matrix_listener.ron during save")?;
        file.write_all(ron.as_bytes())
            .context("Unable to write matrix_listener.ron while saving")?;
        trace!("Saved Session!");
        Ok(())
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