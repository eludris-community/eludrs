use serde::{Deserialize, Serialize};
use todel::models::{ErrorResponse, Message};

/// Error type alias
pub type Error<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub(crate) enum MessageResponse {
    Message(Message),
    Error(ErrorResponse),
}
