use std::num::NonZeroU64;
use std::sync::Arc;
use dashmap::DashMap;
use dashmap::mapref::one::{Ref, RefMut};
use crate::player::Player;
use crate::reference::{Reference};
use crate::rest::RestClient;
use crate::Shared;

pub(crate) struct PlayerManager {
    http: RestClient,
    pub(crate) players: DashMap<u64, Arc<Player>>
}

impl PlayerManager {
    pub fn new(http: RestClient) -> Self {
        Self {
            http,
            players: DashMap::new()
        }
    }

    pub fn get_or_insert(&self, guild: u64) -> Reference<Player> {
        if self.players.contains_key(&guild) {
            self.players.get(&guild).unwrap().into()
        } else {
            let player = Player::new(self.http.clone(), NonZeroU64::new(guild).unwrap());
            self.players.insert(guild, Arc::new(player));
            self.players.get(&guild).unwrap().into()
        }
    }

    pub fn get(&self, guild: u64) -> Option<Reference<Player>> {
        self.players.get(&guild).map(|player| player.into())
    }
}