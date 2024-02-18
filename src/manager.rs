use dashmap::DashMap;
use dashmap::mapref::one::{Ref, RefMut};
use crate::player::Player;

pub(crate) struct PlayerManager {
    players: DashMap<u64, Player>
}

impl PlayerManager {
    pub fn new() -> Self {
        Self {
            players: DashMap::new()
        }
    }

    pub fn get_or_insert(&self, guild: u64) -> Ref<u64, Player> {
        if self.players.contains_key(&guild) {
            self.players.get(&guild).unwrap()
        } else {
            let player = Player::new();
            self.players.insert(guild, player);
            self.players.get(&guild).unwrap()
        }
    }

    pub fn get_or_insert_mut(&self, guild: u64) -> RefMut<u64, Player> {
        if self.players.contains_key(&guild) {
            self.players.get_mut(&guild).unwrap()
        } else {
            let player = Player::new();
            self.players.insert(guild, player);
            self.players.get_mut(&guild).unwrap()
        }
    }
}