use thiserror::Error;

#[derive(Debug, Error)]
#[error(transparent)]
pub(crate) enum SocketError {
    Tungstenite(#[from] tokio_tungstenite::tungstenite::Error),
    Deserialize(#[from] serde_json::Error)
}