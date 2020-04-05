use failure::Fail;
use std::io;

#[derive(Fail, Debug)]
pub enum KvsError {
    /// IO error.
    #[fail(display = "{}", _0)]
    Io(#[cause] io::Error),
    /// Serialization or deserialization error.
    #[fail(display = "{}", _0)]
    Serde(#[cause] serde_json::Error),
    /// Removing non-existent key error.
    #[fail(display = "Key not found")]
    KeyNotFound,
    #[fail(display = "{}", _0)]
    Clap(#[cause] clap::Error),
}

impl From<io::Error> for KvsError {
    fn from(err: io::Error) -> KvsError {
        KvsError::Io(err)
    }
}

impl From<serde_json::Error> for KvsError {
    fn from(err: serde_json::Error) -> KvsError {
        KvsError::Serde(err)
    }
}

impl From<clap::Error> for KvsError {
    fn from(err: clap::Error) -> KvsError {
        KvsError::Clap(err)
    }
}

/// Custom Result type to wrap all errors,
/// which possible during work with KvStore
pub type Result<T> = std::result::Result<T, KvsError>;
