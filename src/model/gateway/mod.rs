pub mod event;
pub mod forward;
pub mod ready;
pub mod state;

use serde::Deserialize;

/// Payloads that can be received from the server
#[derive(Deserialize)]
#[serde(tag = "op", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum IncomingPayload {
    Ready(ready::Ready),
    Forward(forward::Forward),
    UpdateState(state::UpdateState),
    Event {
        guild_id: u64,
        event: event::Event
    }
}