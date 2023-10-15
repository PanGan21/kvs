// #![deny(missing_docs)]
//! A simple key/value store.

mod engines;
mod errors;
mod kv;

pub use engines::KvsEngine;
pub use errors::{KvsError, Result};
pub use kv::KvStore;
