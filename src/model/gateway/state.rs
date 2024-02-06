use std::num::NonZeroU64;

use serde::Deserialize;

/// Voice update state related events.
#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum UpdateState {
    /// The server has connected to a voice channel.
    ConnectGateway(ConnectionData),
    /// The server has reconnected to a voice channel after a network issue.
    ReconnectGateway(ConnectionData),
    /// The server has been disconnected from a voice channel, either manually,
    /// an user has kicked or moved it.
    DisconnectGateway(DisconnectData)
}

/// The data about the connection
#[derive(Debug, Deserialize)]
pub struct ConnectionData {
    /// Channel id the server is connected to.
    pub channel_id: Option<NonZeroU64>,
    /// Guild id the server is connected to.
    pub guild_id: NonZeroU64,
    /// The session id of the connection.
    pub session_id: String,
    /// The server nightingale is connected to.
    pub server: String,
    /// The ssrc of the connection.
    pub ssrc: u32
}

#[derive(Debug, Deserialize)]
pub struct DisconnectData {
    /// The channel id the server disconnected from.
    pub channel_id: Option<NonZeroU64>,
    /// The guild id the server disconnected from.
    pub guild_id: NonZeroU64,
    /// The session id of the previous connection.
    pub session_id: String
}
