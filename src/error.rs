use reqwest::StatusCode;
use thiserror::Error;
use crate::model::error::ErrorResponse;

#[derive(Debug, Error)]
#[error(transparent)]
pub(crate) enum SocketError {
    Tungstenite(#[from] tokio_tungstenite::tungstenite::Error),
    Deserialize(#[from] serde_json::Error)
}

#[derive(Error, Debug)]
#[error("{0}")]
pub struct StatusCodeError(pub StatusCode);

#[derive(Debug, Error)]
pub enum HttpError {
    #[error("Server returned an error message: {0:?}")]
    ErrorMessage(#[from] ErrorResponse),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error("Server responded with an unexpected status code: {0:?}")]
    UnexpectedStatus(#[from] StatusCodeError)
}
