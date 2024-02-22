mod commands;

use std::env;
use std::error::Error;
use std::num::NonZeroU64;
use std::sync::Arc;
use futures::StreamExt;
use nightingale_client::config::Config;
use nightingale_client::events::{EventForwarder, IncomingEvent};
use nightingale_client::NightingaleClient;
use tokio::sync::RwLock;
use tracing::{error, info, Level};
use twilight_gateway::stream;
use twilight_gateway::stream::ShardEventStream;
use twilight_http::Client;
use twilight_model::gateway::event::Event;
use twilight_model::id::Id;
use vesper::prelude::{after, DefaultCommandResult, Framework};
use crate::commands::{join, leave, pause, play, resume};
use twilight_gateway::Config as TwilightConfig;
use twilight_model::gateway::Intents;
use vesper::context::SlashContext;

pub struct Shared {
    pub nightingale: RwLock<NightingaleClient>
}

pub type ArcShared = Arc<Shared>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>>{
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    dotenvy::dotenv()?;

    let token = env::var("TOKEN")?;
    let app_id = env::var("APP_ID")?.parse::<u64>()?;

    let http = Arc::new(Client::builder()
        .token(token.clone())
        .build());
    let config = TwilightConfig::new(token, Intents::all());

    let mut shards = stream::create_recommended(&http, config, |_, s| s.build())
        .await?.collect::<Vec<_>>();

    let mut ng = NightingaleClient::new_twilight(Config {
        host: String::from("localhost"),
        port: 8081,
        password: String::from("asupersafepassword"),
        ssl: false,
        user_id: Some(NonZeroU64::new(app_id).unwrap()),
        shards: Some(shards.len() as _),
        connection_attempts: 5
    }, shards.iter());

    ng.connect().await?;

    let forwarder = ng.events_forwarder();

    let s = Arc::new(Shared {
        nightingale: RwLock::new(ng)
    });

    let f = Arc::new(Framework::builder(http.clone(), Id::new(app_id), s.clone())
        .command(play)
        .command(join)
        .command(leave)
        .command(pause)
        .command(resume)
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
    match event {
        Event::Ready(_) => {
            //framework.register_guild_commands(Id::new(<Guild Id>)).await.unwrap();
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
