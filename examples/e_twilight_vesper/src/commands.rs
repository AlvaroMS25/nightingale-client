use lazy_static::lazy_static;
use nightingale_client::source::{Link, Youtube};
use regex::Regex;
use tracing::info;
use vesper::context::SlashContext;
use vesper::framework::DefaultError;
use vesper::prelude::{check, command, DefaultCommandResult};
use crate::ArcShared;

lazy_static! {
    static ref URL_REGEX: Regex = Regex::new(r"https?://(www\.)?[-a-zA-Z0-9@:%._\+~#=]{2,256}\.[a-z]{2,4}\b([-a-zA-Z0-9@:%_\+.~#?&//=]*)").unwrap();
}

async fn send_text(ctx: &SlashContext<'_, ArcShared>, content: String) -> DefaultCommandResult {
    ctx.interaction_client.update_response(&ctx.interaction.token)
        .content(Some(&content))?
        .await?;

    Ok(())
}

#[check]
async fn player_available(ctx: &SlashContext<ArcShared>) -> Result<bool, DefaultError> {
    let p = ctx.data.nightingale.read().await
        .get_player(ctx.interaction.guild_id.unwrap())
        .is_some();
    info!("Player available?: {p}");
    Ok(p)
}

#[command]
#[checks(player_available)]
#[description = "Plays from the specified source or query"]
#[only_guilds]
pub async fn play(
    ctx: &mut SlashContext<ArcShared>,
    #[description = "Query or link to play from"]
    source: String,
) -> DefaultCommandResult {
    ctx.defer(false).await?;

    let client = ctx.data.nightingale.read().await;

    let src = if URL_REGEX.is_match(&source) {
        source
    } else {
        let mut results = client.search(source.clone(), Youtube).await?;

        if results.is_empty() {
            send_text(ctx, format!("No results were found for query: {source}")).await?;
            return Ok(());
        }

        results.remove(0).url
    };

    let track = client.get_player_mut(ctx.interaction.guild_id.unwrap())
        .unwrap()
        .enqueue(Link(src))
        .await?;
    send_text(ctx, format!("Playing {}", track.title.unwrap()).to_string()).await?;

    Ok(())
}

#[command]
#[description = "Joins the specified channel"]
#[only_guilds]
pub async fn join(
    ctx: &SlashContext<ArcShared>,
) -> DefaultCommandResult
{
    let vs = ctx.data.cache.voice_state(
        ctx.interaction.author_id().unwrap(),
        ctx.interaction.guild_id.unwrap()
    ).unwrap();

    ctx.defer(false).await?;

    ctx.data.nightingale.read().await
        .join(vs.guild_id(), vs.channel_id())
        .await?;

    send_text(ctx, "Joined channel!".to_string()).await
}

#[command]
#[checks(player_available)]
#[description = "Leaves the channel"]
#[only_guilds]
pub async fn leave(ctx: &SlashContext<ArcShared>) -> DefaultCommandResult {
    ctx.defer(false).await?;

    ctx.data.nightingale.read()
        .await
        .leave(ctx.interaction.guild_id.unwrap())
        .await?;

    send_text(ctx, "Left channel!".to_string()).await
}

#[command]
#[checks(player_available)]
#[description = "Pauses playback"]
#[only_guilds]
pub async fn pause(ctx: &SlashContext<ArcShared>) -> DefaultCommandResult {
    ctx.defer(false).await?;

    ctx.data.nightingale.read()
        .await
        .get_player_mut(ctx.interaction.guild_id.unwrap())
        .unwrap()
        .pause()
        .await?;

    send_text(ctx, "Player paused!".to_string()).await
}

#[command]
#[checks(player_available)]
#[description = "Resumes playback"]
#[only_guilds]
pub async fn resume(ctx: &SlashContext<ArcShared>) -> DefaultCommandResult {
    ctx.defer(false).await?;

    ctx.data.nightingale.read()
        .await
        .get_player_mut(ctx.interaction.guild_id.unwrap())
        .unwrap()
        .resume()
        .await?;

    send_text(ctx, "Player resumed!".to_string()).await
}

#[command]
#[checks(player_available)]
#[description = "Changes the player volume"]
#[only_guilds]
async fn set_volume(
    ctx: &SlashContext<ArcShared>,
    #[description = "The new volume"] volume: u8
) -> DefaultCommandResult {
    ctx.defer(false).await?;

    ctx.data.nightingale.read()
        .await
        .get_player_mut(ctx.interaction.guild_id.unwrap())
        .unwrap()
        .set_volume(volume)
        .await?;

    send_text(ctx, format!("Set volume to {volume}")).await
}
