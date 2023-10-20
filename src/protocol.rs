use serde::{Deserialize, Serialize};

/// Represents the various types of requests that can be sent from a client to a key-value store server.
///
/// Requests include operations like getting a value for a given key, setting a key-value pair, or removing a key.
#[derive(Deserialize, Debug, Serialize)]
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

/// Represents the response to a 'Get' request from the key-value store server.
///
/// The response can either be successful with an optional value or an error message.
#[derive(Deserialize, Serialize, Debug)]
pub enum GetResponse {
    /// Successful response containing an optional value associated with the requested key.
    Ok(Option<String>),
    /// Error response with a message indicating the reason for the failure.
    Err(String),
}

/// Represents the response to a 'Set' request from the key-value store server.
///
/// The response can either be successful or an error message.
#[derive(Deserialize, Serialize, Debug)]
pub enum SetResponse {
    /// Successful response indicating that the key-value pair has been successfully set.
    Ok(()),
    /// Error response with a message indicating the reason for the failure.
    Err(String),
}

/// Represents the response to a 'Remove' request from the key-value store server.
///
/// The response can either be successful or an error message.
#[derive(Deserialize, Serialize, Debug)]
pub enum RemoveResponse {
    /// Successful response indicating that the key and its associated value have been successfully removed.
    Ok(()),
    /// Error response with a message indicating the reason for the failure.
    Err(String),
}
