use lazy_static::lazy_static;
use nightingale_client::source::{Link, Youtube};
use regex::Regex;
use tracing::info;
use crate::{AnyError, ArcShared};

type Context<'a> = poise::Context<'a, ArcShared, AnyError>;

lazy_static! {
    static ref URL_REGEX: Regex = Regex::new(r"https?://(www\.)?[-a-zA-Z0-9@:%._\+~#=]{2,256}\.[a-z]{2,4}\b([-a-zA-Z0-9@:%_\+.~#?&//=]*)").unwrap();
}

async fn player_available(ctx: Context<'_>) -> Result<bool, AnyError> {
    let p = ctx.data().nightingale.read().await
        .get_player(ctx.guild_id().unwrap())
        .is_some();

    info!("Player available?: {p}");
    Ok(p)
}

/// Joins your channel
#[poise::command(prefix_command, slash_command)]
pub async fn join(ctx: Context<'_>) -> Result<(), AnyError> {
    let (guild, channel) = {
        let guild = ctx.guild().unwrap();
        let voice = guild.voice_states.get(&ctx.author().id)
            .unwrap();

        (guild.id, voice.channel_id.unwrap())
    };

    if let poise::Context::Application(c) = &ctx {
        c.defer().await?;
    }

    ctx.data()
        .nightingale
        .read()
        .await
        .join(guild, channel)
        .await?;

    ctx.reply("Player resumed!").await?;
    Ok(())
}

/// Leaves the current voice channel
#[poise::command(prefix_command, slash_command, check = "player_available")]
pub async fn leave(ctx: Context<'_>) -> Result<(), AnyError> {
    if let Context::Application(c) = &ctx {
        c.defer().await?;
    }

    ctx.data()
        .nightingale
        .read()
        .await
        .leave(ctx.guild_id().unwrap())
        .await?;

    ctx.reply("Left channel!").await?;

    Ok(())
}

/// Pauses playback
#[poise::command(prefix_command, slash_command, check = "player_available")]
pub async fn pause(ctx: Context<'_>) -> Result<(), AnyError> {
    if let poise::Context::Application(c) = &ctx {
        c.defer().await?;
    }

    ctx.data()
        .nightingale
        .read()
        .await
        .get_player_mut(ctx.guild_id().unwrap())
        .unwrap()
        .pause()
        .await?;

    ctx.reply("Player paused!").await?;

    Ok(())
}

/// Resumes playback
#[poise::command(prefix_command, slash_command, check = "player_available")]
pub async fn resume(ctx: Context<'_>) -> Result<(), AnyError> {
    if let poise::Context::Application(c) = &ctx {
        c.defer().await?;
    }

    ctx.data()
        .nightingale
        .read()
        .await
        .get_player_mut(ctx.guild_id().unwrap())
        .unwrap()
        .resume()
        .await?;

    ctx.reply("Player resumed!").await?;

    Ok(())
}

/// Plays from the specified url, or searches
/// for the query and plays the first result
#[poise::command(prefix_command, slash_command, check = "player_available")]
pub async fn play(
    ctx: Context<'_>,
    #[rest]
    #[description = "Query or link to play from"]
    source: String
) -> Result<(), AnyError> {
    if let poise::Context::Application(c) = &ctx {
        c.defer().await?;
    }

    let client = ctx.data().nightingale.read().await;

    let src = if URL_REGEX.is_match(&source) {
        source
    } else {
        let mut results = client.search(source.clone(), Youtube).await?;

        if results.is_empty() {
            ctx.reply(format!("No results were found for query: {source}")).await?;
            return Ok(());
        }

        results.remove(0).url
    };

    let track = client.get_player_mut(ctx.guild_id().unwrap())
        .unwrap()
        .enqueue(Link(src))
        .await?;

    ctx.reply(format!("Playing {}", track.title.unwrap())).await?;

    Ok(())
}
