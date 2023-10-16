// #![deny(missing_docs)]
//! A simple key/value store.

mod engines;
mod errors;
mod server;

pub use engines::{KvStore, KvsEngine};
pub use errors::{KvsError, Result};
pub use server::KvsServer;
