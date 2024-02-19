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

pub struct NightingaleClient {
    socket: SocketHandle,
    http: RestClient,
    config: Config,
    session: Arc<RwLock<Uuid>>,
    players: Arc<PlayerManager>
}

impl NightingaleClient {
    #[cfg(feature = "serenity")]
    pub fn new_serenity(config: Config, handler: impl EventHandler + 'static) -> Self {
        let session = Arc::new(RwLock::new(Uuid::nil()));
        let manager = Arc::new(PlayerManager::new());
        let events = Arc::new(handler) as Arc<dyn EventHandler>;

        Self {
            socket: Socket::new(
                config.clone(),
                session.clone(),
                manager.clone(),
                events
            ),
            http: RestClient::new(session.clone()),
            config,
            session,
            players: manager
        }
    }

    pub async fn connect(&mut self) {

    }
}
