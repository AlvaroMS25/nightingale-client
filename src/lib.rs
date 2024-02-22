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
mod serenity_ext;

#[cfg(feature = "twilight")]
mod stream;

use std::collections::HashMap;
use std::num::NonZeroU64;
use std::sync::Arc;
use dashmap::mapref::one::{Ref, RefMut};
use parking_lot::RwLock;
use tokio_tungstenite::tungstenite::Error;
use twilight_gateway::Shard;
use uuid::Uuid;
use socket::Socket;
use config::Config;
use crate::error::HttpError;
use crate::events::EventForwarder;
use crate::manager::PlayerManager;
use crate::msg::{FromSocketMessage, ToSocketMessage};
use crate::rest::RestClient;
use crate::socket::SocketHandle;

#[cfg(feature = "serenity")]
use crate::events::EventHandler;
use crate::player::Player;
use crate::source::SearchSource;
#[cfg(feature = "twilight")]
use crate::stream::EventStream;

pub(crate) struct Shared {
    pub session: RwLock<Uuid>,
    pub config: RwLock<Config>
}

pub struct NightingaleClient {
    socket: SocketHandle,
    http: RestClient,
    shared: Arc<Shared>,
    players: Arc<PlayerManager>,
}

impl NightingaleClient {
    #[cfg(feature = "serenity")]
    pub fn new_serenity(config: Config, handler: impl EventHandler + 'static) -> Self {
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
    pub fn new_twilight<'a, I>(config: Config, shards: I) -> Self
    where
        I: IntoIterator<Item = &'a Shard>
    {
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

    /// Connects to the server using the provided config.
    pub async fn connect(&mut self) -> Result<(), Error> {
        self.connect_reconnect_inner(ToSocketMessage::Connect).await
    }

    pub async fn disconnect(&mut self) {
        self.socket.sender.send(ToSocketMessage::Disconnect).unwrap();
    }

    pub async fn reconnect(&mut self) -> Result<(), Error> {
        self.connect_reconnect_inner(ToSocketMessage::Reconnect).await
    }

    #[cfg(feature = "twilight")]
    pub fn events(&self) -> Option<EventStream> {
        EventStream::new(&self.socket.events)
    }

    #[cfg(feature = "twilight")]
    pub fn events_forwarder(&self) -> EventForwarder {
        EventForwarder {
            sender: self.socket.sender.clone()
        }
    }

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

    pub async fn leave<G: Into<NonZeroU64>>(&self, guild: G)
        -> Result<(), HttpError> {
        let guild = guild.into();
        self.http.disconnect(guild).await
            .map(|res| {
                self.players.players.remove(&guild.get());
                res
            })
    }

    pub async fn search<S>(&self, query: String, source: S) -> Result<Vec<S::Track>, HttpError>
    where
        S: SearchSource
    {
        self.http.search(query, source).await
    }

    pub async fn playlist<S>(&self, playlist: String, source: S) -> Result<S::Playlist, HttpError>
    where
        S: SearchSource
    {
        self.http.playlist(playlist, source).await
    }

    pub fn get_player(&self, guild: impl Into<NonZeroU64>) -> Option<Ref<u64, Player>> {
        self.players.players.get(&guild.into().get())
    }

    pub fn get_player_mut(&self, guild: impl Into<NonZeroU64>) -> Option<RefMut<u64, Player>> {
        self.players.players.get_mut(&guild.into().get())
    }
}
