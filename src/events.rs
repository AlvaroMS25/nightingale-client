use std::num::NonZeroU64;
use std::sync::Arc;
use dashmap::mapref::one::RefMut;
use crate::model::gateway::ready::Ready;
#[cfg(feature = "serenity")]
use crate::model::gateway::event::{TrackEnd, TrackErrored};
#[cfg(feature = "serenity")]
use crate::model::gateway::state::{ConnectionData, DisconnectData};
#[cfg(feature = "serenity")]
use crate::model::track::Track;
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
#[cfg(feature = "serenity")]
use serenity::all::{VoiceServerUpdateEvent, VoiceState};
use tracing::error;
use crate::error::HttpError;
use crate::manager::PlayerManager;
use crate::model::connection::PartialConnectionInfo;
use crate::rest::RestClient;
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
    /// Triggered when the client disconnects from the voice server.
    async fn on_server_disconnect(&self, error: tokio_tungstenite::tungstenite::Error) {}
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
    },
    /// Received when the socket disconnects from the server.
    DisconnectedFromServer(tokio_tungstenite::tungstenite::Error),
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

#[derive(Clone)]
pub struct EventForwarder {
    pub(crate) players: Arc<PlayerManager>
}

impl EventForwarder {
    #[cfg(feature = "twilight")]
    /// Forwards an event to the server. This call does not forward the full event to the server,
    /// instead it only uses the minimum required information by the server.
    pub async fn forward(&self, event: &TwilightEvent) {
        let res = match event {
            TwilightEvent::VoiceServerUpdate(su) => {
                self._server_update(
                    su.guild_id.get(),
                    su.endpoint.clone(),
                    su.token.clone()
                ).await
            },
            TwilightEvent::VoiceStateUpdate(su) => {
                let Some(guild) = su.guild_id else { return; };
                self._state_update(
                    guild.get(),
                    su.channel_id.map(|c| c.into_nonzero()),
                    su.session_id.clone()
                ).await
            },
            _ => return
        };

        if let Err(e) = res {
            error!("Error while forwarding voice event to server: {e:?}")
        }
    }

    #[cfg(feature = "serenity")]
    pub async fn voice_server_update(&self, event: VoiceServerUpdateEvent) -> Result<(), HttpError> {
        if event.guild_id.is_none() {
            return Ok(());
        }

        self._server_update(event.guild_id.unwrap().get(), event.endpoint, event.token).await
    }

    #[cfg(feature = "serenity")]
    pub async fn voice_state_update(&self, vs: VoiceState) -> Result<(), HttpError> {
        if vs.guild_id.is_none() {
            return Ok(());
        }

        self._state_update(
            vs.guild_id.unwrap().get(),
            vs.channel_id.map(Into::into),
            vs.session_id
        ).await
    }

    async fn _server_update(
        &self,
        guild: u64,
        endpoint: Option<String>,
        token: String
    ) -> Result<(), HttpError> {
        let mut p = self.players.get_or_insert(guild);
        let mut partial = p.partial.lock();

        partial.endpoint = endpoint;
        partial.token = Some(token);

        self.update_if_needed(&p.http, p.guild, &mut partial).await
    }

    async fn _state_update(
        &self,
        guild: u64,
        channel: Option<NonZeroU64>,
        session: String
    ) -> Result<(), HttpError> {
        let mut p = self.players.get_or_insert(guild);
        let mut partial = p.partial.lock();

        partial.channel_id = channel;
        partial.session_id = Some(session);

        self.update_if_needed(&p.http, p.guild, &mut partial).await
    }

    async fn update_if_needed(
        &self,
        http: &RestClient,
        guild: NonZeroU64,
        partial: &mut PartialConnectionInfo
    ) -> Result<(), HttpError> {
        if partial.complete() {
            let info = std::mem::take(partial).into_info();

            http.update_player(guild, Some(info)).await?;
        }

        Ok(())
    }
}
