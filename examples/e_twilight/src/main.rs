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
use twilight_gateway::Config as TwilightConfig;
use twilight_model::gateway::Intents;

pub struct Shared {
    pub nightingale: RwLock<NightingaleClient>,
    pub cache: InMemoryCache,
    pub http: Client,
}

pub type ArcShared = Arc<Shared>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>>{
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    dotenvy::dotenv()?;

    let token = env::var("TOKEN")?;

    let http = Client::builder()
        .token(token.clone())
        .build();
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
        cache: InMemoryCacheBuilder::default().build(),
        http,
    });

    let mut stream = ShardEventStream::new(shards.iter_mut());
    let read = s.nightingale.read().await;

    let mut voice_events = read.events().unwrap();

    loop {
        tokio::select! {
            Some((_id, Ok(ev))) = stream.next() => {
                handle_gateway_event(&s, ev, &forwarder).await;
            }
            Some(ev) = voice_events.next() => {
                handle_voice_event(ev).await;
            }
        }
    }
}

async fn handle_gateway_event(
    shared: &ArcShared,
    event: Event,
    f: &EventForwarder
) {
    shared.cache.update(&event);

    match event {
        Event::Ready(_) => {
            info!("Ready!");
        },
        Event::MessageCreate(i) => {
            let c = shared.clone();

            tokio::spawn(async move {
                if let Err(e) = commands::execute(c, i.0).await {
                    error!("Failed to execute command: {e}");
                }
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
