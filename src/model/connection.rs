use std::mem::MaybeUninit;
use std::num::NonZeroU64;
use serde::Serialize;

/// Connection information used to connect to a voice channel
#[derive(Serialize, Clone, Debug)]
pub struct ConnectionInfo {
    /// Channel id to connect to.
    pub channel_id: Option<NonZeroU64>,
    /// Endpoint to connect to.
    pub endpoint: String,
    /// Session id of the connection.
    pub session_id: String,
    /// Token of the connection.
    pub token: String
}
