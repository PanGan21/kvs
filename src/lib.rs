// #![deny(missing_docs)]
//! A simple key/value store.

mod errors;
mod kv;

pub use errors::Result;
pub use kv::KvStore;
