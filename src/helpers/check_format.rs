use log::debug;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CheckFormatError {
    #[error("Format {0} is not supported")]
    FormatNotSupported(String),
}

pub fn check_format(format: &Option<String>) -> Result<(), CheckFormatError> {
    match format {
        Some(v) => {
            if v != "org.matrix.custom.html" {
                debug!("Message parsed properly, but format {} is unsupported so no conversion is taking place.", v);
                Err(CheckFormatError::FormatNotSupported(v.to_string()))
            } else {
                Ok(())
            }
        }
        None => Ok(()),
    }
}
