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
use crate::manager::PlayerManager;

pub struct NightingaleKey;

impl TypeMapKey for NightingaleKey {
    type Value = Arc<RwLock<NightingaleClient>>;
}

#[async_trait]
pub trait SerenityExt {
    /// Registers a new instance of the client into the client's typemap
    fn register_nightingale(
        self,
        config: Config,
        event_handler: impl EventHandler + 'static
    ) -> Self;
    /// Registers an existing instance into the client's typemap
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
        self.voice_manager_arc(instance.voice_manager())
            .type_map_insert::<NightingaleKey>(Arc::new(RwLock::new(instance)))
    }
}

pub(crate) struct NightingaleVoiceManager {
    pub shared: Arc<Shared>,
    pub players: Arc<PlayerManager>
}

#[async_trait]
impl VoiceGatewayManager for NightingaleVoiceManager {
    async fn initialise(&self, shard_count: u32, user_id: UserId) {
        let mut cfg = self.shared.config.write();

        cfg.shards = shard_count as _;
        cfg.user_id = user_id.into();
    }

    async fn register_shard(&self, shard_id: u32, sender: UnboundedSender<ShardRunnerMessage>) {
        self.shared.shards.shards.insert(shard_id as _, sender);
    }

    async fn deregister_shard(&self, shard_id: u32) {
        self.shared.shards.shards.remove(&(shard_id as _));
    }

    async fn server_update(&self, guild_id: GuildId, endpoint: &Option<String>, token: &str) {
        let mut p = self.players.get_or_insert_mut(guild_id.get());

        p.info.endpoint = endpoint.clone();
        p.info.token = Some(token.to_string());

        let _ = p.update_state().await;
    }

    async fn state_update(&self, guild_id: GuildId, voice_state: &VoiceState) {
        let mut p = self.players.get_or_insert_mut(guild_id.get());

        p.info.channel_id = voice_state.channel_id.map(Into::into);
        p.info.session_id = Some(voice_state.session_id.clone());

        let _ = p.update_state().await;
    }
}
