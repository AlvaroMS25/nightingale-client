use futures::channel::mpsc::UnboundedSender;
use serde_json::Value;

#[cfg(feature = "serenity")]
use serenity::all::ShardRunnerMessage;

pub(crate) enum ToSocketMessage {
    Connect,
    Disconnect,
    Reconnect,
    Resume,
    Kill
}

pub(crate) enum FromSocketMessage {
    ConnectedSuccessfully,
    Disconnected,
    FailedToConnect(tokio_tungstenite::tungstenite::Error),
    FailedToResume,
}
