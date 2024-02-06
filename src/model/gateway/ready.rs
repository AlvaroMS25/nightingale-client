use serde::Deserialize;
use uuid::Uuid;

/// The ready event, fired when a new connection is established
/// with the server.
#[derive(Debug, Deserialize)]
pub struct Ready {
    /// Whether if the session was resumed or not.
    pub resumed: bool,
    /// The session id itself.
    pub session: Uuid
}