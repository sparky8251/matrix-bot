//! Helper function used to verify the formatting of the recieved message is processable by later steps

use log::error;
use thiserror::Error;

#[derive(Error, Debug)]
/// Type used to represent the error state
pub enum CheckFormatError {
    #[error("Format {0} is not supported")]
    /// Returned if supplied format is not supported
    ///
    /// Has related `Display` implementation for ease of error reporting to the admin
    FormatNotSupported(String),
}

#[derive(Debug)]
/// Type used to represent various success states
pub enum CheckFormatSuccess {
    /// Returned if supplied format can be processed by later steps
    FormatSupported(String),
    /// Returned if no format was supplied
    NoFormat,
}

/// Checks supplied format and returns `Ok(_)` if it can be processed in later steps and `Err(e)` if it can't.
pub fn check_format(format: &Option<String>) -> Result<CheckFormatSuccess, CheckFormatError> {
    match format {
        Some(v) => {
            if v != "org.matrix.custom.html" {
                error!("Message parsed properly, but format {} is unsupported so no conversion is taking place.", v);
                Err(CheckFormatError::FormatNotSupported(v.to_string()))
            } else {
                Ok(CheckFormatSuccess::FormatSupported(v.to_string()))
            }
        }
        None => Ok(CheckFormatSuccess::NoFormat),
    }
}
