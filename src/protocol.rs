use serde::{Deserialize, Serialize};

/// Represents the various types of requests that can be sent from a client to a key-value store server.
///
/// Requests include operations like getting a value for a given key, setting a key-value pair, or removing a key.
#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    /// Request to get the value associated with a specific key.
    Get {
        /// The key for which to retrieve the value.
        key: String,
    },
    /// Request to set a key-value pair in the store.
    Set {
        /// The key for the key-value pair.
        key: String,
        /// The value to associate with the key.
        value: String,
    },
    /// Request to remove a key and its associated value from the store.
    Remove {
        /// The key to be removed.
        key: String,
    },
}

/// Represents the various types of responses that can be sent from a server to a key-value store client.
///
/// Responses include operations like getting a value for a given key, setting a key-value pair, or removing a key.
#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    /// Represents the response to a 'Get' request from the key-value store server.
    ///
    /// The response can either be successful with an optional value or an error message.
    Get(Option<String>),
    /// Represents the response to a 'Set' request from the key-value store server.
    ///
    /// The response can either be successful or an error message.
    Set,
    /// Represents the response to a 'Remove' request from the key-value store server.
    ///
    /// The response can either be successful or an error message.
    Remove,
    /// Error response with a message indicating the reason for the failure.
    Err(String),
}
