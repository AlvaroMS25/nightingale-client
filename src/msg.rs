use serde_json::Value;
use crate::config::Config;

pub(crate) enum ToSocketMessage {
    Connect,
    Disconnect,
    Reconnect,
    Resume,
    Send(Value),
    Kill
}

pub(crate) enum FromSocketMessage {
    ConnectedSuccessfully,
    Disconnected,
    FailedToConnect(tokio_tungstenite::tungstenite::Error),
    FailedToResume,
}
