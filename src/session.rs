use std::collections::HashMap;
use std::fs::{self, File};
use std::time::{Duration, SystemTime};

use anyhow::Result;
use log::trace;
use ruma_client::{identifiers::RoomId, Session};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SavedSession {
    username: String,
    password: String,
    session: Option<Session>,
    last_sync: Option<String>,
    active_room: Option<RoomId>,
    last_txn_id: u64,
    last_correction_time: HashMap<RoomId, SystemTime>,
}

impl SavedSession {
    pub fn load_session() -> Result<Self> {
        let file = File::open("session_data.ron")?;
        Ok(ron::de::from_reader(file)?)
    }

    pub fn save_session(&self) -> Result<()> {
        fs::write("session_data.ron", ron::ser::to_string(&self)?)?;
        trace!("Saved!");
        Ok(())
    }

    pub fn get_username(&self) -> String {
        self.username.clone()
    }

    pub fn get_password(&self) -> String {
        self.password.clone()
    }

    pub fn get_session(&self) -> Option<Session> {
        self.session.clone()
    }

    pub fn get_last_sync(&self) -> Option<String> {
        self.last_sync.clone()
    }

    pub fn set_last_sync(&mut self, last_sync: Option<String>) {
        self.last_sync = last_sync;
    }

    pub fn set_session(&mut self, session: Session) {
        self.session = Some(session)
    }

    pub fn set_last_correction_time(&mut self, room_id: &RoomId, time: SystemTime) {
        self.last_correction_time.insert(room_id.clone(), time);
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
            None => true, // Will only be None if this session has not yet corrected anyone, so return true to allow correction
        }
    }
}
