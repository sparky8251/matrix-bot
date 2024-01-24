use native_db::{native_db, InnerKeyValue};
use native_model::{native_model, Model};
use serde::Deserialize;
use serde::Serialize;
use std::fmt::{Display, Formatter};

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug)]
#[native_model(id = 1, version = 1)]
#[native_db]
pub struct AccessToken {
    #[primary_key]
    pub(crate) id: u8,
    #[secondary_key(unique)]
    pub(crate) access_token: String,
}

impl Display for AccessToken {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.access_token)
    }
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug)]
#[native_model(id = 1, version = 1)]
#[native_db]
pub struct LastSync {
    #[primary_key]
    pub(crate) id: u8,
    #[secondary_key(unique)]
    pub(crate) last_sync: String,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug)]
#[native_model(id = 1, version = 1)]
#[native_db]
pub struct CorrectionTimeCooldown {
    #[primary_key]
    pub(crate) room_id: String,
    #[secondary_key]
    pub(crate) last_correction_time: u64,
}
