use std::num::NonZeroU64;
use dashmap::DashMap;
use dashmap::mapref::one::{Ref, RefMut};
use crate::player::Player;
use crate::rest::RestClient;

pub(crate) struct PlayerManager {
    http: RestClient,
    players: DashMap<u64, Player>
}

impl PlayerManager {
    pub fn new(http: RestClient) -> Self {
        Self {
            http,
            players: DashMap::new()
        }
    }

    pub fn get_or_insert(&self, guild: u64) -> Ref<u64, Player> {
        if self.players.contains_key(&guild) {
            self.players.get(&guild).unwrap()
        } else {
            let player = Player::new(self.http.clone(), NonZeroU64::new(guild).unwrap());
            self.players.insert(guild, player);
            self.players.get(&guild).unwrap()
        }
    }

    pub fn get_or_insert_mut(&self, guild: u64) -> RefMut<u64, Player> {
        if self.players.contains_key(&guild) {
            self.players.get_mut(&guild).unwrap()
        } else {
            let player = Player::new(self.http.clone(), NonZeroU64::new(guild).unwrap());
            self.players.insert(guild, player);
            self.players.get_mut(&guild).unwrap()
        }
    }
}