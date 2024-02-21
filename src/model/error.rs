use serde::Deserialize;
use thiserror::Error;

#[derive(Deserialize, Debug, Error)]
#[error("Server responded with an error: {message}")]
pub struct ErrorResponse {
    pub message: String
}


