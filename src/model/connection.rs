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

#[derive(Default)]
pub(crate) struct PartialConnectionInfo {
    pub channel_id: Option<NonZeroU64>,
    pub endpoint: Option<String>,
    pub session_id: Option<String>,
    pub token: Option<String>
}

impl PartialConnectionInfo {
    pub fn complete(&self) -> bool {
        self.endpoint.is_some()
            && self.session_id.is_some()
            && self.token.is_some()
    }

    pub fn into_info(self) -> ConnectionInfo {
        assert!(self.complete());

        ConnectionInfo {
            channel_id: self.channel_id,
            endpoint: self.endpoint.unwrap(),
            session_id: self.session_id.unwrap(),
            token: self.token.unwrap()
        }
    }
}
