pub mod model;
pub mod config;
mod error;
mod socket;
mod player;
mod rest;
mod msg;
#[cfg(feature = "serenity")]
mod events;
mod manager;

use std::cell::UnsafeCell;
use std::sync::Arc;
use parking_lot::RwLock;
use uuid::Uuid;
use socket::Socket;
use config::Config;
use crate::events::EventHandler;
use crate::manager::PlayerManager;
use crate::rest::RestClient;
use crate::socket::SocketHandle;

pub(crate) struct Shared {
    pub players: PlayerManager,
    pub session: RwLock<Uuid>,
    pub config: RwLock<Config>
}

pub struct NightingaleClient {
    socket: SocketHandle,
    http: RestClient,
    shared: Arc<Shared>
}

impl NightingaleClient {
    #[cfg(feature = "serenity")]
    pub fn new_serenity(config: Config, handler: impl EventHandler + 'static) -> Self {
        let events = Arc::new(handler) as Arc<dyn EventHandler>;
        let shared = Arc::new(Shared {
            players: PlayerManager::new(),
            session: RwLock::new(Uuid::nil()),
            config: RwLock::new(config)
        });

        Self {
            socket: Socket::new(
                Arc::clone(&shared),
                events
            ),
            http: RestClient::new(Arc::clone(&shared)),
            shared
        }
    }

    pub async fn connect(&mut self) {

    }
}
