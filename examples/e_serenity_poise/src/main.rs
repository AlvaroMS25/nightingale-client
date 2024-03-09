use std::env;
use std::error::Error;
use std::sync::Arc;
use nightingale_client::config::Config;
use nightingale_client::events::EventHandler;
use nightingale_client::NightingaleClient;
use poise::async_trait;
use poise::serenity_prelude::{ClientBuilder, GatewayIntents, GuildId, Http};
use tokio::sync::RwLock;
use tracing::{error, Level};

mod commands;

pub type AnyError = Box<dyn Error + Send + Sync>;

pub struct Shared {
    pub nightingale: RwLock<NightingaleClient>
}

pub type ArcShared = Arc<Shared>;

#[tokio::main]
async fn main() -> Result<(), AnyError> {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    dotenvy::dotenv()?;
    let token = env::var("TOKEN")?;
    let guild = env::var("GUILD")?.parse::<u64>()?;
    let http = Http::new(&token).get_current_application_info().await?;

    let mut ng = NightingaleClient::new_serenity(Config {
        user_id: http.id.into(),
        ..Default::default()
    }, VoiceEvents);

    ng.connect().await?;

    let voice_manager = ng.voice_manager();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                commands::join(),
                commands::leave(),
                commands::pause(),
                commands::resume(),
                commands::play()
            ],
            ..Default::default()
        })
        .setup(move |ctx, _, f| Box::pin(async move {
            poise::builtins::register_in_guild(ctx, &f.options().commands, GuildId::new(guild)).await?;

            let shared = Arc::new(Shared {
                nightingale: RwLock::new(ng)
            });

            Ok(shared)
        }))
        .build();

    let mut client = ClientBuilder::new(token, GatewayIntents::all())
        .voice_manager_arc(voice_manager)
        .framework(framework)
        .await?;

    if let Err(e) = client.start().await {
        error!("Error while running client, {e}");
    }

    Ok(())
}

struct VoiceEvents;

#[async_trait]
impl EventHandler for VoiceEvents {

}
