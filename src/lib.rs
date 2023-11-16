#![deny(missing_docs)]
//! A simple key/value store.

mod client;
mod engines;
mod errors;
mod protocol;
mod server;
/// The thread pool implementation
pub mod thread_pool;

pub use client::KvsClient;
pub use engines::{KvStore, KvsEngine, SledKvsEngine};
pub use errors::{KvsError, Result};
pub use protocol::{GetResponse, RemoveResponse, Request, SetResponse};
pub use server::KvsServer;
