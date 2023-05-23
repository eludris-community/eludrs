use serde::{Deserialize, Serialize};
use todel::models::{ErrorResponse, Message};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub(crate) enum MessageResponse {
    Message(Message),
    Error(ErrorResponse),
}
