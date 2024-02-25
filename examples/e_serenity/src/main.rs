mod commands;

use std::env;
use std::error::Error;
use nightingale_client::config::Config;
use serenity::all::{Context, EventHandler, GatewayIntents, Ready, StandardFramework};
use serenity::{async_trait, Client};
use tracing::{error, info, Level};
use nightingale_client::events::EventHandler as VoiceEventHandler;
use nightingale_client::serenity_ext::{NightingaleKey, SerenityExt};
use serenity::framework::standard::Configuration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>>{
    dotenvy::dotenv()?;

    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    let token = env::var("TOKEN")?;

    let framework = StandardFramework::new()
        .group(&commands::MUSIC_GROUP);

    framework.configure(Configuration::new().prefix("!"));

    let intents = GatewayIntents::all();

    let nightingale_config = Config {
        host: "localhost".to_string(),
        port: 8081,
        password: "mypassword".to_string(),
        ssl: false,
        user_id: None, // User id and shards will be set by the client, so no worry
        shards: None,
        connection_attempts: 5,
    };

    let mut client = Client::builder(&token, intents)
        .framework(framework)
        .event_handler(GatewayEvents)
        .register_nightingale(nightingale_config, VoiceEvents)
        .await
        .unwrap();

    if let Err(e) = client.start().await {
        error!("Client had an error: {e}");
    }

    Ok(())
}

struct GatewayEvents;

#[async_trait]
impl EventHandler for GatewayEvents {
    async fn ready(&self, ctx: Context, _: Ready) {
        let data = ctx.data.read().await;
        let client = data.get::<NightingaleKey>().expect("Set at startup");

        client.write().await
            .connect()
            .await
            .expect("Failed to connect to server");

        info!("Ready!");
    }
}

struct VoiceEvents;

#[async_trait]
impl VoiceEventHandler for VoiceEvents {
    async fn on_ready(&self, ready: nightingale_client::model::gateway::ready::Ready) {
        info!("[Voice Event] Ready! Session: {}", ready.session);
    }
}
