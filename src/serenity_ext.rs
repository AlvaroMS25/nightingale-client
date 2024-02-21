use std::num::NonZeroU64;
use std::sync::Arc;
use futures::channel::mpsc::UnboundedSender;
use serde_json::json;
use serenity::all::{GuildId, ShardRunnerMessage, UserId, VoiceState};
use serenity::async_trait;
use serenity::client::ClientBuilder;
use serenity::gateway::VoiceGatewayManager;
use typemap_rev::TypeMapKey;
use crate::config::Config;
use crate::events::EventHandler;
use crate::msg::ToSocketMessage;
use crate::NightingaleClient;

impl TypeMapKey for NightingaleClient {
    type Value = Arc<Self>;
}

#[async_trait]
pub trait SerenityExt {
    fn register_nightingale(
        self,
        config: Config,
        event_handler: impl EventHandler + 'static
    ) -> Self;
    fn register_nightingale_from_instance(self, instance: Arc<NightingaleClient>) -> Self;
}

#[async_trait]
impl SerenityExt for ClientBuilder {
    fn register_nightingale(
        self,
        config: Config,
        event_handler: impl EventHandler + 'static
    ) -> Self {
        let this = Arc::new(NightingaleClient::new_serenity(config, event_handler));

        self.register_from_instance(this)
    }

    fn register_nightingale_from_instance(self, instance: Arc<NightingaleClient>) -> Self {
        self.voice_manager_arc(instance.clone())
            .type_map_insert::<NightingaleClient>(instance)
    }
}

#[async_trait]
impl VoiceGatewayManager for NightingaleClient {
    async fn initialise(&self, shard_count: u32, user_id: UserId) {
        let mut cfg = self.shared.config.write();

        cfg.shards = Some(shard_count as _);
        cfg.user_id = Some(user_id.into());
    }

    async fn register_shard(&self, shard_id: u32, sender: UnboundedSender<ShardRunnerMessage>) {
        self.socket.sender.send(ToSocketMessage::RegisterShard(shard_id, sender)).unwrap();
    }

    async fn deregister_shard(&self, shard_id: u32) {
        self.socket.sender.send(ToSocketMessage::DeregisterShard(shard_id)).unwrap()
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

        self.socket.sender.send(ToSocketMessage::Send(value)).unwrap();
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

        self.socket.sender.send(ToSocketMessage::Send(value)).unwrap();
    }
}
