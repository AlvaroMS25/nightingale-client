use std::num::NonZeroU64;
use std::sync::Arc;
use crate::model::gateway::ready::Ready;
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
use crate::model::gateway::{event::Event, state::UpdateState, IncomingPayload};
#[cfg(feature = "twilight")]
use crate::msg::ToSocketMessage;
#[cfg(feature = "twilight")]
use twilight_model::gateway::event::Event as TwilightEvent;
#[cfg(feature = "twilight")]
use serde_json::json;
use crate::manager::PlayerManager;
use crate::Shared;

#[cfg(feature = "serenity")]
/// Trait defining what events can be fired from the server.
#[async_trait]
pub trait EventHandler: Send + Sync {
    /// Triggered after connecting to the server.
    async fn on_ready(&self, _ready: Ready) {}
    /// Triggered when the server connects to a voice channel.
    async fn on_gateway_connect(&self, _info: ConnectionData) {}
    /// Triggered when the server reconnects to a voice channel due to some network issues.
    async fn on_gateway_reconnect(&self, _info: ConnectionData) {}
    /// Triggered when the server disconnects or gets disconnected from a voice channel, this
    /// includes channel moves and forceful disconnects from users.
    async fn on_gateway_disconnect(&self, _info: DisconnectData) {}
    /// Triggered when a track has started its playback.
    async fn on_track_start(&self, _player: &Player, _track: Track) {}
    /// Triggered when a track finished its playback.
    async fn on_track_end(&self, _player: &Player, _track_end: TrackEnd) {}
    /// Triggered when a track encountered an error when trying to play.
    async fn on_track_errored(&self, _player: &Player, _track_errored: TrackErrored) {}
}

#[cfg(feature = "twilight")]
/// All possible incoming events from the server.
pub enum IncomingEvent {
    /// Received after connecting to the server.
    Ready(Ready),
    /// Received when a voice state change occurs.
    UpdateState(UpdateState),
    /// Received when a playback related event occurs.
    Event {
        /// The guild id the event belongs to.
        guild_id: u64,
        /// The event itself.
        event: Event
    }
}

#[cfg(feature = "twilight")]
impl From<IncomingPayload> for IncomingEvent {
    fn from(value: IncomingPayload) -> Self {
        match value {
            IncomingPayload::Ready(r) => Self::Ready(r),
            IncomingPayload::UpdateState(s) => Self::UpdateState(s),
            IncomingPayload::Event { guild_id, event } => Self::Event { guild_id, event }
        }
    }
}

#[cfg(feature = "twilight")]
pub struct EventForwarder {
    pub(crate) players: Arc<PlayerManager>
}

#[cfg(feature = "twilight")]
impl EventForwarder {
    /// Forwards an event to the server. This call does not forward the full event to the server,
    /// instead it only uses the minimum required information by the server.
    pub async fn forward(&self, event: &TwilightEvent) {
        let p = match event {
            TwilightEvent::VoiceServerUpdate(su) => {
                self.server_update(
                    su.guild_id.get(),
                    su.endpoint.clone(),
                    su.token.clone()
                ).await
            },
            TwilightEvent::VoiceStateUpdate(su) => {
                let Some(guild) = su.guild_id else { return; };
                self.state_update(
                    guild.get(),
                    su.channel_id.map(|c| c.into_nonzero()),
                    su.session_id.clone()
                ).await;
            },
            _ => return
        };
    }

    async fn state_update(&self, guild: u64, channel_id: Option<NonZeroU64>, session_id: String) {
        let mut p = self.players.get_or_insert_mut(guild);

        p.info.channel_id = channel_id;
        p.info.session_id = Some(session_id);

        let _ = p.update_state().await;
    }

    async fn server_update(&self, guild: u64, endpoint: Option<String>, token: String) {
        let mut p = self.players.get_or_insert_mut(guild);

        p.info.endpoint = endpoint;
        p.info.token = Some(token);

        let _ = p.update_state().await;
    }
}
