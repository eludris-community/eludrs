use serde::{Deserialize, Serialize};
use todel::models::Message;

/// Error type alias
pub type Error<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct RatelimitResponse {
    pub(crate) data: RatelimitData,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct RatelimitData {
    pub(crate) retry_after: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub(crate) enum MessageResponse {
    Message(Message),
    Ratelimited(RatelimitResponse),
}
