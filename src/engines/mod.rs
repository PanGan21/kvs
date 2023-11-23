use crate::Result;
use async_trait::async_trait;

/// Trait for a key value storage engine.
#[async_trait]
pub trait KvsEngine: Clone + Send + 'static {
    /// Set the value of a string key to a string.
    /// Return an error if the value is not written successfully.
    async fn set(self, key: String, value: String) -> Result<()>;

    /// Get the string value of a string key. If the key does not exist, return None.
    /// Return an error if the value is not read successfully.
    async fn get(self, key: String) -> Result<Option<String>>;

    /// Remove a given string key.
    /// Return an error if the key does not exit or value is not read successfully.
    async fn remove(self, key: String) -> Result<()>;
}

mod kvs;
mod sled;

pub use kvs::KvStore;
pub use sled::SledKvsEngine;
