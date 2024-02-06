use serde::Deserialize;
use serde_json::Value;

/// The forward event, fired when nightingale requests
/// a payload to be forwarded directly to discord's gateway.
#[derive(Debug, Deserialize)]
pub struct Forward {
    /// The shard that should forward the payload.
    pub shard: u64,
    /// The payload that should be forwarded
    pub payload: Value
}