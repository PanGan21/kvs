use std::io;

use thiserror::Error;

/// Custom error type for the Key-Value Store.
#[derive(Debug, Error)]
pub enum KvsError {
    /// IO error.
    #[error("{}", _0)]
    Io(#[from] io::Error),

    /// Serialization or Deserialization error.
    #[error("{}", _0)]
    Serde(#[from] serde_json::Error),

    /// Remove non existing key.
    #[error("Key not found")]
    KeyNotFound,

    /// Unexpected command type error.
    #[error("Unexpected command type")]
    UnexpectedCommandType,

    /// Error with a string message
    #[error("{}", _0)]
    StringError(String),
}

/// Result type for kvs.
pub type Result<T> = std::result::Result<T, KvsError>;
