use std::fs::{self, File};

use log::info;
use ruma_client::Session;
use ruma_identifiers::RoomId;
use serde::{Deserialize, Serialize};

use crate::error::Result;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SavedSession {
    username: String,
    password: String,
    session: Option<Session>,
    last_sync: Option<String>,
    active_room: Option<RoomId>,
    last_txn_id: u64,
}

impl SavedSession {
    pub fn load_session() -> Result<Self> {
        let file = File::open("session_data.ron")?;
        Ok(ron::de::from_reader(file)?)
    }

    pub fn save_session(&self) -> Result<()> {
        fs::write("session_data.ron", ron::ser::to_string(&self)?)?;
        info!("Saved!");
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

    // FIXME: This needs to be an idempotent/unique ID to be spec compliant
    pub fn next_txn_id(&mut self) -> String {
        self.last_txn_id += 1;
        self.last_txn_id.to_string()
    }
}
