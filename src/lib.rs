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
mod stream;
pub mod reference;

use std::num::NonZeroU64;
use std::sync::Arc;
use parking_lot::RwLock;
use tokio_tungstenite::tungstenite::Error;
use uuid::Uuid;
use socket::Socket;
use config::Config;
use crate::error::HttpError;
use crate::manager::PlayerManager;
use crate::msg::{FromSocketMessage, ToSocketMessage};
use crate::rest::RestClient;
use crate::socket::SocketHandle;

#[cfg(feature = "serenity")]
use crate::events::EventHandler;
#[cfg(feature = "serenity")]
use serenity::gateway::VoiceGatewayManager;
#[cfg(feature = "serenity")]
use crate::serenity_ext::NightingaleVoiceManager;

use crate::player::Player;
use crate::source::SearchSource;
#[cfg(feature = "twilight")]
use crate::stream::EventStream;
#[cfg(feature = "twilight")]
use crate::events::EventForwarder;
#[cfg(feature = "twilight")]
use twilight_gateway::Shard;
#[cfg(feature = "twilight")]
use std::collections::HashMap;

use crate::reference::{Reference, ReferenceMut};

pub(crate) struct Shared {
    pub session: RwLock<Uuid>,
    pub config: RwLock<Config>
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
        assert_ne!(config.user_id.get(), 1);
        let events = Arc::new(handler) as Arc<dyn EventHandler>;
        let shared = Arc::new(Shared {
            session: RwLock::new(Uuid::nil()),
            config: RwLock::new(config)
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
    pub fn new_twilight<'a, I>(config: Config, shards: I) -> Self
    where
        I: IntoIterator<Item = &'a Shard>
    {
        assert_ne!(config.user_id.get(), 1);
        let map = shards.into_iter().map(|s| (s.id().number(), s.sender()))
            .collect::<HashMap<_, _>>();

        let shared = Arc::new(Shared {
            session: RwLock::new(Uuid::nil()),
            config: RwLock::new(config)
        });

        let rest = RestClient::new(shared.clone());
        let players = Arc::new(PlayerManager::new(rest.clone()));

        Self {
            socket: Socket::new(
                shared.clone(),
                players.clone(),
                map
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
            sender: self.socket.sender.clone()
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
        EventStream::new(&self.socket.events)
    }

    #[cfg(feature = "twilight")]
    /// Returns a forwarder that must be used to forward voice server update and voice state update
    /// events, this will only send the minimum required fields in the payload, not the whole event.
    pub fn events_forwarder(&self) -> EventForwarder {
        EventForwarder {
            sender: self.socket.sender.clone()
        }
    }

    /// Joins the given voice channel.
    pub async fn join<G, C>(&self, guild: G, channel: C)
        -> Result<(), HttpError>
    where
        G: Into<NonZeroU64>,
        C: Into<NonZeroU64>
    {
        let guild = guild.into();
        self.http.connect(guild.into(), channel.into()).await
            .map(|res| {
                self.players.get_or_insert(guild.get());
                res
            })
    }

    /// Leaves the given voice channel.
    pub async fn leave<G: Into<NonZeroU64>>(&self, guild: G)
        -> Result<(), HttpError> {
        let guild = guild.into();
        self.http.disconnect(guild).await
            .map(|res| {
                self.players.players.remove(&guild.get());
                res
            })
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
    pub fn get_player(&self, guild: impl Into<NonZeroU64>) -> Option<Reference<Player>> {
        self.players.players.get(&guild.into().get())
            .map(Into::into)
    }

    /// Returns a mutable reference to the player of the provided guild, if present.
    pub fn get_player_mut(&self, guild: impl Into<NonZeroU64>) -> Option<ReferenceMut<Player>> {
        self.players.players.get_mut(&guild.into().get())
            .map(Into::into)
    }
}
