pub mod model;
pub mod config;
mod error;
mod socket;
mod player;
mod rest;
mod msg;
mod manager;
mod source;
mod events;
#[cfg(feature = "serenity")]
mod serenity_ext;

#[cfg(feature = "twilight")]
mod stream;

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use tokio_tungstenite::tungstenite::Error;
use twilight_gateway::Shard;
use uuid::Uuid;
use socket::Socket;
use config::Config;
use crate::manager::PlayerManager;
use crate::msg::{FromSocketMessage, ToSocketMessage};
use crate::rest::RestClient;
use crate::socket::SocketHandle;

#[cfg(feature = "serenity")]
use crate::events::EventHandler;
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
    pub fn new_twilight<'a, I>(&self, config: Config, shards: I) -> Self
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

    pub fn events(&self) -> Option<EventStream> {
        EventStream::new(&self.socket.events)
    }
}
