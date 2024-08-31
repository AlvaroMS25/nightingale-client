use std::num::NonZeroU64;
use std::sync::Arc;

use futures::SinkExt;
use parking_lot::RwLock;
#[cfg(feature = "serenity")]
use serenity::gateway::VoiceGatewayManager;
use songbird::ConnectionInfo;
use tokio_tungstenite::tungstenite::Error;
use uuid::Uuid;

use config::Config;
use socket::Socket;

use crate::config::SessionConfig;
use crate::error::HttpError;
use crate::events::EventForwarder;
#[cfg(feature = "serenity")]
use crate::events::EventHandler;
use crate::manager::PlayerManager;
use crate::msg::{FromSocketMessage, ToSocketMessage};
use crate::player::Player;
use crate::reference::{Reference};
use crate::rest::RestClient;
#[cfg(feature = "serenity")]
use crate::serenity_ext::NightingaleVoiceManager;
use crate::socket::SocketHandle;
use crate::source::SearchSource;
#[cfg(feature = "twilight")]
use crate::stream::EventStream;

pub mod model;
pub mod config;
pub mod error;
mod socket;
pub mod player;
pub mod rest;
mod msg;
mod manager;
pub mod source;
pub mod events;
#[cfg(feature = "serenity")]
pub mod serenity_ext;

#[cfg(feature = "twilight")]
pub mod stream;
pub mod reference;
mod reflock;

pub(crate) struct Shared {
    pub session: RwLock<Uuid>,
    pub config: RwLock<Config>,
    pub session_config: RwLock<SessionConfig>,
}

/// Client that handles a single connection to a nightingale server.
pub struct NightingaleClient {
    socket: SocketHandle,
    http: RestClient,
    shared: Arc<Shared>,
    players: Arc<PlayerManager>,
}

impl NightingaleClient {
    #[cfg(feature = "serenity")]
    /// Creates a new instance to be used with serenity.
    pub fn new_serenity(config: Config, handler: impl EventHandler + 'static) -> Self {
        let events = Arc::new(handler) as Arc<dyn EventHandler>;
        let shared = Arc::new(Shared {
            session: RwLock::new(Uuid::nil()),
            config: RwLock::new(config),
            session_config: RwLock::new(SessionConfig::default()),
        });
        let rest = RestClient::new(shared.clone());
        let players = Arc::new(PlayerManager::new(rest.clone()));

        Self {
            socket: Socket::new(
                Arc::clone(&shared),
                players.clone(),
                events
            ),
            http: rest,
            shared,
            players
        }
    }

    #[cfg(feature = "twilight")]
    /// Creates a new instance to be used with twilight.
    pub fn new_twilight(config: Config) -> Self {
        assert!(config.user_id.is_some());

        let shared = Arc::new(Shared {
            session: RwLock::new(Uuid::nil()),
            config: RwLock::new(config),
            session_config: RwLock::new(SessionConfig::default()),
        });

        let rest = RestClient::new(shared.clone());
        let players = Arc::new(PlayerManager::new(rest.clone()));

        Self {
            socket: Socket::new(
                shared.clone(),
                players.clone(),
            ),
            http: rest,
            shared,
            players
        }
    }

    #[cfg(feature = "serenity")]
    /// Returns a voice manager to be used with [`ClientBuilder#event_handler_arc`]
    ///
    /// [`ClientBuilder#event_handler_arc`]: serenity::all::ClientBuilder::event_handler_arc
    pub fn voice_manager(&self) -> Arc<dyn VoiceGatewayManager> {
        Arc::new(NightingaleVoiceManager {
            shared: self.shared.clone(),
            players: self.players.clone()
        })
    }

    pub fn rest(&self) -> &RestClient {
        &self.http
    }

    async fn connect_reconnect_inner(&mut self, p: ToSocketMessage) -> Result<(), Error> {
        self.socket.sender.send(p).unwrap();
        while let Some(msg) = self.socket.receiver.recv().await {
            match msg {
                FromSocketMessage::ConnectedSuccessfully => return Ok(()),
                FromSocketMessage::FailedToConnect(e) => return Err(e),
                _ => continue
            }
        }

        Ok(())
    }

    /// Connects to the server.
    pub async fn connect(&mut self) -> Result<(), Error> {
        self.connect_reconnect_inner(ToSocketMessage::Connect).await
    }

    /// Disconnects from the server.
    pub async fn disconnect(&mut self) {
        self.socket.sender.send(ToSocketMessage::Disconnect).unwrap();
    }

    /// Reconnects to the server.
    pub async fn reconnect(&mut self) -> Result<(), Error> {
        self.connect_reconnect_inner(ToSocketMessage::Reconnect).await
    }

    #[cfg(feature = "twilight")]
    /// Returns an event stream that can be used to listen for events coming from the server.
    ///
    /// A single instance of the event stream can be present at a time. If called when there is
    /// another stream present, this will return `None`, after dropping the other stream this method
    /// will return `Some` again
    pub fn events(&self) -> Option<EventStream> {
        EventStream::new(self.socket.events.clone())
    }

    /// Returns a forwarder that must be used to forward voice server update and voice state update
    /// events, this will only send the minimum required fields in the payload, not the whole event.
    pub fn events_forwarder(&self) -> EventForwarder {
        EventForwarder {
            players: self.players.clone()
        }
    }

    /// Joins the given voice channel.
    pub async fn create_player(
        &self,
        info: impl Into<ConnectionInfo>
    ) -> Result<Arc<Player>, HttpError>
    {
        let info = info.into();
        let guild = info.guild_id.0;

        tracing::info!("[Sending http request] Creating player");

        self.http.update_player(guild, Some(model::connection::ConnectionInfo {
            channel_id: info.channel_id.map(|c| c.0),
            endpoint: info.endpoint,
            session_id: info.session_id,
            token: info.token
        })).await?;

        tracing::info!("[Request sent] Player created on server");
        tracing::info!("[Local player] Creating player on client");

        let player = self.players.get_or_insert(guild.get()).into_owned();

        tracing::info!("[Local player] Created player on client");

        Ok(player)
    }

    /// Leaves the given voice channel.
    pub async fn destroy_player<G: Into<NonZeroU64>>(&self, guild: G) -> Result<(), HttpError> {
        let guild = guild.into();
        self.players.players.remove(&guild.get());
        self.http.update_player(guild, None).await?;
        Ok(())
    }

    /// Makes a search on the provided source.
    pub async fn search<S>(&self, query: String, source: S) -> Result<Vec<S::Track>, HttpError>
    where
        S: SearchSource
    {
        self.http.search(query, source).await
    }

    /// Gets the playlist items from the specified source.
    pub async fn playlist<S>(&self, playlist: String, source: S) -> Result<S::Playlist, HttpError>
    where
        S: SearchSource
    {
        self.http.playlist(playlist, source).await
    }

    /// Returns a reference to the player of the provided guild, if present.
    pub fn get_player(&self, guild: impl Into<NonZeroU64>) -> Option<Arc<Player>> {
        self.players.get(guild.into().get())
            .map(Reference::into_owned)
    }

    pub fn get_player_ref(&self, guild: impl Into<NonZeroU64>) -> Option<Reference<Player>> {
        self.players.get(guild.into().get())
    }
}
