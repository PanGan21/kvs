use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum KvsError {
    /// IO error.
    #[error("{}", _0)]
    Io(#[from] io::Error),
    /// Serialization or Deserialization error.
    #[error("{}", _0)]
    Serde(#[from] serde_json::Error),
    /// Unexpected command type error.
    #[error("Unexpected command type")]
    UnexpectedCommandType,
}

/// Result type for kvs.
pub type Result<T> = std::result::Result<T, KvsError>;
