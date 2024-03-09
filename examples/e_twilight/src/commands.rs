use std::error::Error;
use lazy_static::lazy_static;
use nightingale_client::source::{Link, Youtube};
use regex::Regex;
use tracing::info;
use twilight_model::channel::Message;
use crate::{ArcShared, Shared};

lazy_static! {
    static ref URL_REGEX: Regex = Regex::new(r"https?://(www\.)?[-a-zA-Z0-9@:%._\+~#=]{2,256}\.[a-z]{2,4}\b([-a-zA-Z0-9@:%_\+.~#?&//=]*)").unwrap();
}

pub async fn execute(
    shared: ArcShared,
    msg: Message
) -> Result<(), Box<dyn Error>> {
    let space = if let Some(idx) = msg.content.find(" ") {
        idx
    } else {
        msg.content.len()
    };

    if !msg.content.starts_with("!") {
        return Ok(());
    }

    let command = &msg.content[1..space];

    let rest = msg.content[space..msg.content.len()].trim().to_string();

    match command {
        "play" => {
            play(&shared, msg, rest).await?;
        },
        "join" => {
            join(&shared, msg).await?;
        },
        "leave" => {
            leave(&shared, msg).await?;
        }
        "pause" => {
            pause(&shared, msg).await?;
        },
        "resume" => {
            resume(&shared, msg).await?;
        },
        "set_volume" => {
            set_volume(&shared, msg, rest).await?;
        }
        _ => ()
    }

    Ok(())
}

async fn send_text(
    shared: &Shared,
    message: &Message,
    content: String
) -> Result<(), Box<dyn Error>> {
    shared.http.create_message(message.channel_id)
        .content(&content)?
        .await
        .map(|_| ())
        .map_err(From::from)
}

async fn player_available(shared: &Shared, msg: &Message) -> bool {
    let p = shared.nightingale.read().await
        .get_player(msg.guild_id.unwrap())
        .is_some();
    info!("Player available?: {p}");

    p
}

async fn play(shared: &Shared, msg: Message, rest: String) -> Result<(), Box<dyn Error>> {
    if !player_available(shared, &msg).await {
        return Ok(())
    }

    let client = shared.nightingale.read().await;

    let src = if URL_REGEX.is_match(&rest) {
        rest
    } else {
        let mut results = client.search(rest.clone(), Youtube).await?;

        if results.is_empty() {
            send_text(
                shared,
                &msg,
                format!("No results were found for query: {}", rest)
            ).await?;
            return Ok(());
        }

        results.remove(0).url
    };

    let track = client.get_player_mut(msg.guild_id.unwrap())
        .unwrap()
        .enqueue(Link(src))
        .await?;

    send_text(shared, &msg, format!("Playing {}", track.title.unwrap()).to_string())
        .await
}

async fn join(shared: &Shared, msg: Message) -> Result<(), Box<dyn Error>> {
    let vs = shared.cache.voice_state(
        msg.author.id,
        msg.guild_id.unwrap()
    ).unwrap();

    shared.nightingale.read().await
        .join(vs.guild_id(), vs.channel_id())
        .await?;


    send_text(shared, &msg, "Joined channel!".to_string()).await
}

async fn leave(shared: &Shared, msg: Message) -> Result<(), Box<dyn Error>> {
    if !player_available(shared, &msg).await {
        return Ok(())
    }

    shared.nightingale.read()
        .await
        .leave(msg.guild_id.unwrap())
        .await?;

    send_text(shared, &msg, "Left channel!".to_string()).await
}

async fn pause(shared: &Shared, msg: Message) -> Result<(), Box<dyn Error>> {
    if !player_available(shared, &msg).await {
        return Ok(())
    }

    shared.nightingale.read()
        .await
        .get_player_mut(msg.guild_id.unwrap())
        .unwrap()
        .pause()
        .await?;

    send_text(shared, &msg, "Player paused!".to_string()).await
}

async fn resume(shared: &Shared, msg: Message) -> Result<(), Box<dyn Error>> {
    if !player_available(shared, &msg).await {
        return Ok(())
    }

    shared.nightingale.read()
        .await
        .get_player_mut(msg.guild_id.unwrap())
        .unwrap()
        .resume()
        .await?;

    send_text(shared, &msg, "Player resumed!".to_string()).await
}

async fn set_volume(shared: &Shared, msg: Message, rest: String) -> Result<(), Box<dyn Error>> {
    if !player_available(shared, &msg).await {
        return Ok(())
    }

    let volume = rest.parse::<u8>()?;

    shared.nightingale.read()
        .await
        .get_player_mut(msg.guild_id.unwrap())
        .unwrap()
        .set_volume(volume)
        .await?;

    send_text(shared, &msg, format!("Set volume to {volume}")).await
}
