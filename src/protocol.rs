use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Serialize)]
pub enum Request {
    Get { key: String },
    Set { key: String, value: String },
    Remove { key: String },
}

#[derive(Deserialize)]
pub enum GetResponse {
    Ok(Option<String>),
    Err(String),
}

pub enum SetResponse {
    Ok(()),
    Err(String),
}

pub enum RemoveResponse {
    Ok(()),
    Err(String),
}
