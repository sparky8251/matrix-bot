//! Helper function used to verify the formatting of the recieved message is processable by later steps

use anyhow::anyhow;
use ruma::events::room::message::MessageFormat;
use std::fmt;
use std::fmt::Formatter;

#[derive(Debug)]
/// Type used to represent the error state
pub enum CheckFormatError {
    /// Returned if supplied format is not supported
    ///
    /// Has related `Display` implementation for ease of error reporting to the admin
    FormatNotSupported(String),
}

impl fmt::Display for CheckFormatError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            CheckFormatError::FormatNotSupported(e) =>
                write!(f, "Message parsed properly, but format {} is not supported so no conversion is taking place.", e),
        }
    }
}

#[derive(Debug)]
/// Type used to represent various success states
pub enum CheckFormatSuccess {
    /// Returned if supplied format can be processed by later steps
    FormatSupported(MessageFormat),
    /// Returned if no format was supplied
    NoFormat,
}

/// Checks supplied format and returns `Ok(_)` if it can be processed in later steps and `Err(e)` if it can't.
pub fn check_format(format: Option<&MessageFormat>) -> anyhow::Result<CheckFormatSuccess> {
    match format {
        Some(MessageFormat::Html) => Ok(CheckFormatSuccess::FormatSupported(MessageFormat::Html)),
        Some(v) => Err(anyhow!(CheckFormatError::FormatNotSupported(v.to_string()))),
        None => Ok(CheckFormatSuccess::NoFormat),
    }
}
