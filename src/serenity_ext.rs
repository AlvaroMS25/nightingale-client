use std::num::NonZeroU64;
use std::sync::Arc;
use futures::channel::mpsc::UnboundedSender;
use serde_json::json;
use serenity::all::{GuildId, ShardRunnerMessage, UserId, VoiceState};
use serenity::async_trait;
use serenity::client::ClientBuilder;
use serenity::gateway::VoiceGatewayManager;
use tokio::sync::RwLock;
use typemap_rev::TypeMapKey;
use crate::config::Config;
use crate::events::EventHandler;
use crate::msg::ToSocketMessage;
use crate::{NightingaleClient, Shared};
use tokio::sync::mpsc::UnboundedSender as TokioSender;

pub struct NightingaleKey;

impl TypeMapKey for NightingaleKey {
    type Value = Arc<RwLock<NightingaleClient>>;
}

#[async_trait]
pub trait SerenityExt {
    fn register_nightingale(
        self,
        config: Config,
        event_handler: impl EventHandler + 'static
    ) -> Self;
    fn register_nightingale_from_instance(self, instance: NightingaleClient) -> Self;
}

#[async_trait]
impl SerenityExt for ClientBuilder {
    fn register_nightingale(
        self,
        config: Config,
        event_handler: impl EventHandler + 'static
    ) -> Self {
        let this = NightingaleClient::new_serenity(config, event_handler);

        self.register_nightingale_from_instance(this)
    }

    fn register_nightingale_from_instance(self, instance: NightingaleClient) -> Self {
        let manager = NightingaleVoiceManager {
            shared: instance.shared.clone(),
            sender: instance.socket.sender.clone()
        };

        self.voice_manager(manager)
            .type_map_insert::<NightingaleKey>(Arc::new(RwLock::new(instance)))
    }
}

struct NightingaleVoiceManager {
    shared: Arc<Shared>,
    sender: TokioSender<ToSocketMessage>
}

#[async_trait]
impl VoiceGatewayManager for NightingaleVoiceManager {
    async fn initialise(&self, shard_count: u32, user_id: UserId) {
        let mut cfg = self.shared.config.write();

        cfg.shards = shard_count as _;
        cfg.user_id = user_id.into();
    }

    async fn register_shard(&self, shard_id: u32, sender: UnboundedSender<ShardRunnerMessage>) {
        self.sender.send(ToSocketMessage::RegisterShard(shard_id, sender)).unwrap();
    }

    async fn deregister_shard(&self, shard_id: u32) {
        self.sender.send(ToSocketMessage::DeregisterShard(shard_id)).unwrap()
    }

    async fn server_update(&self, guild_id: GuildId, endpoint: &Option<String>, token: &str) {
        let value = json!({
            "op": "update_voice_server",
            "data": {
                "guild_id": guild_id.get(),
                "endpoint": endpoint,
                "token": token
            }
        });

        self.sender.send(ToSocketMessage::Send(value)).unwrap();
    }

    async fn state_update(&self, guild_id: GuildId, voice_state: &VoiceState) {
        let value = json!({
            "op": "update_voice_state",
            "data": {
                "guild_id": guild_id.get(),
                "user_id": voice_state.user_id.get(),
                "session_id": &voice_state.session_id,
                "channel_id": voice_state.channel_id.map(|c| c.get())
            }
        });

        self.sender.send(ToSocketMessage::Send(value)).unwrap();
    }
}
