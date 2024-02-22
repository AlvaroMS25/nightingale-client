use std::sync::Arc;
use nightingale_client::source::{Link, Youtube};
use tracing::info;
use vesper::context::SlashContext;
use vesper::framework::DefaultError;
use vesper::parsers::VoiceChannelId;
use vesper::prelude::{check, command, DefaultCommandResult};
use crate::ArcShared;


async fn send_text(cx: &SlashContext<'_, ArcShared>, content: String) -> DefaultCommandResult {
    cx.interaction_client.update_response(&cx.interaction.token)
        .content(Some(&content))?
        .await?;

    Ok(())
}

#[check]
async fn player_available(cx: &SlashContext<ArcShared>) -> Result<bool, DefaultError> {
    let p = cx.data.nightingale.read().await
        .get_player(cx.interaction.guild_id.as_ref().unwrap().into_nonzero())
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
    #[description = "Link for the song"]
    link: Option<String>,
    #[description = "Query to search from, will play first match"]
    query: Option<String>
) -> DefaultCommandResult {
    let mut link = link;
    let mut query = query;
    ctx.defer(false).await?;

    if link.is_none() && query.is_none() {
        send_text(ctx, "No input provided".to_string()).await?;
        return Ok(())
    }

    if link.is_some() && query.is_some() {
        query.take();
    }

    let read = ctx.data.nightingale.read().await;

    if query.is_some() {
        info!("Searching on youtube...");
        send_text(ctx, "Searching on youtube and playing...".to_string()).await?;
        let mut tracks = read.search(query.unwrap(), Youtube).await?;

        if tracks.len() == 0 {
            send_text(ctx, "No results found".to_string()).await?;
            return Ok(());
        }

        let first = tracks.remove(0);
        info!("First: {}", first.title);

        link = Some(tracks.remove(0).url);
    }

    let mut p = read.get_player_mut(ctx.interaction.guild_id.as_ref().unwrap().into_nonzero()).unwrap();

    p.enqueue(Link(link.unwrap())).await?;
    send_text(ctx, "Playing...".to_string()).await?;

    Ok(())
}

#[command]
#[description = "Joins the specified channel"]
#[only_guilds]
pub async fn join(
    cx: &SlashContext<ArcShared>,
    #[description = "Channel to join"] channel: VoiceChannelId
) -> DefaultCommandResult
{
    cx.defer(false).await?;
    cx.data.nightingale.read().await
        .join(cx.interaction.guild_id.as_ref().unwrap().into_nonzero(), channel.into_nonzero())
        .await?;


    send_text(cx, "Joined voice channel".to_string()).await
}

#[command]
#[checks(player_available)]
#[description = "Leaves the channel"]
#[only_guilds]
pub async fn leave(cx: &SlashContext<ArcShared>) -> DefaultCommandResult {
    cx.defer(false).await?;

    cx.data.nightingale.read()
        .await
        .leave(cx.interaction.guild_id.as_ref().unwrap().into_nonzero())
        .await?;

    send_text(cx, "Left channel!".to_string()).await
}

#[command]
#[checks(player_available)]
#[description = "Pauses playback"]
#[only_guilds]
pub async fn pause(cx: &SlashContext<ArcShared>) -> DefaultCommandResult {
    cx.defer(false).await?;

    cx.data.nightingale.read()
        .await
        .get_player_mut(cx.interaction.guild_id.as_ref().unwrap().into_nonzero())
        .unwrap()
        .pause()
        .await?;

    send_text(cx, "Paused player!".to_string()).await
}

#[command]
#[checks(player_available)]
#[description = "Resumes playback"]
#[only_guilds]
pub async fn resume(cx: &SlashContext<ArcShared>) -> DefaultCommandResult {
    cx.defer(false).await?;

    cx.data.nightingale.read()
        .await
        .get_player_mut(cx.interaction.guild_id.as_ref().unwrap().into_nonzero())
        .unwrap()
        .resume()
        .await?;

    send_text(cx, "Resumed player!".to_string()).await
}
