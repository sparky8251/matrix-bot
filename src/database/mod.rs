use anyhow::{Context, Result};
use native_db::db_type::Error;
use native_db::transaction::RwTransaction;
use native_db::Input;

pub mod models;

pub fn insert_or_update<T: Input + Clone>(rw: &RwTransaction, old: T, new: T) -> Result<()> {
    match rw.insert(new.clone()) {
        Ok(_) => (),
        Err(e) => match e {
            Error::DuplicateKey { .. } => rw.update(old, new).context("Unable to update db")?,
            _ => return Err(e).context("Unable to update db"),
        },
    };
    Ok(())
}
