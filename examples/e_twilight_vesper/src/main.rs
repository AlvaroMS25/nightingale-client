mod commands;

use std::env;
use std::error::Error;
use std::sync::Arc;
use futures::StreamExt;
use nightingale_client::config::Config;
use nightingale_client::events::{EventForwarder, IncomingEvent};
use nightingale_client::NightingaleClient;
use tokio::sync::RwLock;
use tracing::{error, info, Level};
use twilight_cache_inmemory::{InMemoryCache, InMemoryCacheBuilder};
use twilight_gateway::stream;
use twilight_gateway::stream::ShardEventStream;
use twilight_http::Client;
use twilight_model::gateway::event::Event;
use twilight_model::id::Id;
use vesper::prelude::{after, DefaultCommandResult, Framework};
use crate::commands::{join, leave, pause, play, resume, set_volume};
use twilight_gateway::Config as TwilightConfig;
use twilight_model::gateway::Intents;
use vesper::context::SlashContext;

pub struct Shared {
    pub nightingale: RwLock<NightingaleClient>,
    pub cache: InMemoryCache
}

pub type ArcShared = Arc<Shared>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>>{
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    dotenvy::dotenv()?;

    let token = env::var("TOKEN")?;

    let http = Arc::new(Client::builder()
        .token(token.clone())
        .build());
    let app_id = http.current_user_application().await?.model().await?.id;
    let config = TwilightConfig::new(token, Intents::all());

    let mut shards = stream::create_recommended(&http, config, |_, s| s.build())
        .await?.collect::<Vec<_>>();

    let mut ng = NightingaleClient::new_twilight(Config {
        user_id: app_id.into(),
        shards: shards.len() as _,
        ..Default::default()
    }, shards.iter());

    ng.connect().await?;

    let forwarder = ng.events_forwarder();

    let s = Arc::new(Shared {
        nightingale: RwLock::new(ng),
        cache: InMemoryCacheBuilder::default().build()
    });

    let f = Arc::new(Framework::builder(http.clone(), app_id, s.clone())
        .command(play)
        .command(join)
        .command(leave)
        .command(pause)
        .command(resume)
        .command(set_volume)
        .after(log_errors)
        .build());

    let mut stream = ShardEventStream::new(shards.iter_mut());
    let read = s.nightingale.read().await;

    let mut voice_events = read.events().unwrap();

    loop {
        tokio::select! {
            Some((_id, Ok(ev))) = stream.next() => {
                handle_gateway_event(&f, ev, &forwarder).await;
            }
            Some(ev) = voice_events.next() => {
                handle_voice_event(ev).await;
            }
        }
    }
}

#[after]
async fn log_errors(_cx: &SlashContext<ArcShared>, name: &str, out: Option<DefaultCommandResult>) {
    if let Err(why) = out.unwrap() {
        error!("Command {name} raised an error; {why:?}");
    }
}

async fn handle_gateway_event(framework: &Arc<Framework<ArcShared>>, event: Event, f: &EventForwarder) {
    framework.data.cache.update(&event);

    match event {
        Event::Ready(_) => {
            let guild = Id::new(env::var("GUILD").expect("Guild not set").parse().expect("Not a number"));
            framework.register_guild_commands(guild).await.unwrap();
            info!("Ready!");
        },
        Event::InteractionCreate(i) => {
            let c = framework.clone();

            tokio::spawn(async move {
                c.process(i.0).await;
            });
        },
        other => f.forward(&other)
    }
}
async fn handle_voice_event(event: IncomingEvent) {
    match event {
        IncomingEvent::Ready(r) => info!("[Voice Event] Ready: {r:?}"),
        IncomingEvent::UpdateState(s) => info!("[Voice Event] Update State; {s:?}"),
        IncomingEvent::Event {event, ..} => info!("[Voice Event] Event: {event:?}")
    }
}
