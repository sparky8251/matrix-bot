use std::io;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    RonDeError(ron::de::Error),
    RonSerError(ron::ser::Error),
    RumaClientError(ruma_client::Error),
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self::IoError(e)
    }
}

impl From<ron::de::Error> for Error {
    fn from(e: ron::de::Error) -> Self {
        Self::RonDeError(e)
    }
}

impl From<ron::ser::Error> for Error {
    fn from(e: ron::ser::Error) -> Self {
        Self::RonSerError(e)
    }
}

impl From<ruma_client::Error> for Error {
    fn from(e: ruma_client::Error) -> Self {
        Self::RumaClientError(e)
    }
}
