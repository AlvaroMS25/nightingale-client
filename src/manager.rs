use std::num::NonZeroU64;
use std::sync::Arc;
use dashmap::DashMap;
use dashmap::mapref::one::{Ref, RefMut};
use crate::player::Player;
use crate::rest::RestClient;
use crate::Shared;

pub(crate) struct PlayerManager {
    http: RestClient,
    shared: Arc<Shared>,
    pub(crate) players: DashMap<u64, Player>
}

impl PlayerManager {
    pub fn new(http: RestClient, shared: Arc<Shared>) -> Self {
        Self {
            http,
            shared,
            players: DashMap::new()
        }
    }

    pub fn get_or_insert(&self, guild: u64) -> Ref<u64, Player> {
        if self.players.contains_key(&guild) {
            self.players.get(&guild).unwrap()
        } else {
            let shard = self.shared.shards.for_guild(guild);
            let player = Player::new(self.http.clone(), NonZeroU64::new(guild).unwrap(), shard);
            self.players.insert(guild, player);
            self.players.get(&guild).unwrap()
        }
    }

    pub fn get_or_insert_mut(&self, guild: u64) -> RefMut<u64, Player> {
        if self.players.contains_key(&guild) {
            self.players.get_mut(&guild).unwrap()
        } else {
            let shard = self.shared.shards.for_guild(guild);
            let player = Player::new(self.http.clone(), NonZeroU64::new(guild).unwrap(), shard);
            self.players.insert(guild, player);
            self.players.get_mut(&guild).unwrap()
        }
    }
}