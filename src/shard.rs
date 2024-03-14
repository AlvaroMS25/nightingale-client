use std::collections::HashMap;
use dashmap::DashMap;
#[cfg(feature = "serenity")]
use futures::channel::mpsc::UnboundedSender as Sender;
#[cfg(feature = "serenity")]
use serenity::all::ShardRunnerMessage;
#[cfg(feature = "twilight")]
use twilight_gateway::MessageSender;

pub struct ShardStorage {
    #[cfg(feature = "serenity")]
    pub shards: DashMap<u64, Sender<ShardRunnerMessage>>,
    #[cfg(feature = "twilight")]
    pub shards: HashMap<u64, MessageSender>
}

impl ShardStorage {
    #[cfg(feature = "serenity")]
    pub fn new() -> Self {
        Self {
            shards: DashMap::new()
        }
    }

    #[cfg(feature = "twilight")]
    pub fn new(shards: HashMap<u64, MessageSender>) -> Self {
        Self {
            shards
        }
    }

    #[cfg(feature = "serenity")]
    pub fn for_guild(&self, guild: u64) -> Sender<ShardRunnerMessage> {
        self.shards.get(&shard_id(guild, self.shards.len() as _))
            .expect("Invalid number of shards provided")
            .value()
            .clone()
    }

    #[cfg(feature = "twilight")]
    pub fn for_guild(&self, guild: u64) -> MessageSender {
        self.shards.get(&shard_id(guild, self.shards.len() as _))
            .expect("Invalid number of shards provided")
            .clone()
    }
}

#[inline]
fn shard_id(guild_id: u64, shard_count: u64) -> u64 {
    (guild_id >> 22) % shard_count
}
