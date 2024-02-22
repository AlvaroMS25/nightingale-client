#[cfg(feature = "serenity")]
use crate::model::gateway::event::{TrackEnd, TrackErrored};
#[cfg(feature = "serenity")]
use crate::model::gateway::state::{ConnectionData, DisconnectData};
#[cfg(feature = "serenity")]
use crate::model::track::Track;
#[cfg(feature = "serenity")]
use crate::player::Player;
#[cfg(feature = "serenity")]
use serenity::async_trait;
#[cfg(feature = "twilight")]
use crate::model::gateway::{ready::Ready, event::Event, state::UpdateState, IncomingPayload};

#[cfg(feature = "serenity")]
#[async_trait]
pub trait EventHandler: Send + Sync {
    async fn on_ready(&self, _ready: Ready) {}
    async fn on_gateway_connect(&self, _info: ConnectionData) {}
    async fn on_gateway_reconnect(&self, _info: ConnectionData) {}
    async fn on_gateway_disconnect(&self, _info: DisconnectData) {}
    async fn on_track_start(&self, _player: &Player, _track: Track) {}
    async fn on_track_end(&self, _player: &Player, _track_end: TrackEnd) {}
    async fn on_track_errored(&self, _player: &Player, _track_errored: TrackErrored) {}
}

pub enum IncomingEvent {
    Ready(Ready),
    UpdateState(UpdateState),
    Event {
        guild_id: u64,
        event: Event
    }
}

impl From<IncomingPayload> for IncomingEvent {
    fn from(value: IncomingPayload) -> Self {
        match value {
            IncomingPayload::Ready(r) => Self::Ready(r),
            IncomingPayload::Forward(_) => unreachable!(),
            IncomingPayload::UpdateState(s) => Self::UpdateState(s),
            IncomingPayload::Event { guild_id, event } => Self::Event { guild_id, event }
        }
    }
}
