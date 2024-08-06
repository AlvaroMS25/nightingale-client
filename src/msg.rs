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
