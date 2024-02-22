use serde_json::json;
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
use tokio::sync::mpsc::UnboundedSender;
#[cfg(feature = "twilight")]
use crate::model::gateway::{ready::Ready, event::Event, state::UpdateState, IncomingPayload};
#[cfg(feature = "twilight")]
use crate::msg::ToSocketMessage;
#[cfg(feature = "twilight")]
use twilight_model::gateway::event::Event as TwilightEvent;

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

pub struct EventForwarder {
    pub(crate) sender: UnboundedSender<ToSocketMessage>
}

impl EventForwarder {
    pub fn forward(&self, event: &TwilightEvent) {
        let p = match event {
            TwilightEvent::VoiceServerUpdate(su) => json!({
                "op": "update_voice_server",
                "data": {
                    "guild_id": su.guild_id.get(),
                    "endpoint": &su.endpoint,
                    "token": &su.token
                }
            }),
            TwilightEvent::VoiceStateUpdate(su) => json!({
                "op": "update_voice_state",
                "data": {
                    "guild_id": su.guild_id.map(|g| g.get()),
                    "user_id": su.user_id.get(),
                    "session_id": &su.session_id,
                    "channel_id": su.channel_id.map(|c| c.get())
                }
            }),
            _ => return
        };

        self.sender.send(ToSocketMessage::Send(p)).unwrap();
    }
}
