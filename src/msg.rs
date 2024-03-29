use futures::channel::mpsc::UnboundedSender;
use serde_json::Value;

#[cfg(feature = "serenity")]
use serenity::all::ShardRunnerMessage;

pub(crate) enum ToSocketMessage {
    Connect,
    Disconnect,
    Reconnect,
    Resume,
    Send(Value),
    #[cfg(feature = "serenity")]
    RegisterShard(u32, UnboundedSender<ShardRunnerMessage>),
    #[cfg(feature = "serenity")]
    DeregisterShard(u32),
    Kill
}

pub(crate) enum FromSocketMessage {
    ConnectedSuccessfully,
    Disconnected,
    FailedToConnect(tokio_tungstenite::tungstenite::Error),
    FailedToResume,
}
