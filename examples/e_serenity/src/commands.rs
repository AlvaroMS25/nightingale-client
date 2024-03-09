use lazy_static::lazy_static;
use nightingale_client::serenity_ext::NightingaleKey;
use nightingale_client::source::{Link, Youtube};
use regex::Regex;
use serenity::all::Message;
use serenity::all::standard::{Args, CommandOptions, CommandResult, Reason};
use serenity::all::standard::macros::{check, command, group};
use serenity::prelude::Context;

lazy_static! {
    static ref URL_REGEX: Regex = Regex::new(r"https?://(www\.)?[-a-zA-Z0-9@:%._\+~#=]{2,256}\.[a-z]{2,4}\b([-a-zA-Z0-9@:%_\+.~#?&//=]*)").unwrap();
}

#[group]
#[commands(join, leave, pause, resume, play)]
pub struct Music;

#[check]
pub async fn exists_player(
    ctx: &Context,
    msg: &Message,
    _: &mut Args,
    _: &CommandOptions
) -> Result<(), Reason>
{
    let map = ctx.data.read().await;
    let client = map.get::<NightingaleKey>().expect("Set on startup");

    let result = client.read().await
        .get_player(msg.guild_id.unwrap())
        .map(|_| ())
        .ok_or_else(|| Reason::User("Player non-existent".to_string()));

    result
}

#[check]
pub async fn voice_connected(
    ctx: &Context,
    msg: &Message,
    _: &mut Args,
    _: &CommandOptions
) -> Result<(), Reason>
{
    let guild = msg.guild(&ctx.cache).unwrap();

    let voice = guild.voice_states.get(&msg.author.id);

    if voice.is_some() {
        Ok(())
    } else {
        Err(Reason::User("User not in voice channel".to_string()))
    }
}

#[command]
#[only_in(guilds)]
#[checks(voice_connected)]
pub async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    let (guild, channel) = {
        let g = msg.guild(&ctx.cache).unwrap();
        let voice = g.voice_states
            .get(&msg.author.id)
            .unwrap();

        (g.id, voice.channel_id.unwrap())
    };

    let map = ctx.data.read().await;
    let client = map.get::<NightingaleKey>().expect("Set on startup");

    client.read().await
        .join(guild, channel)
        .await?;

    Ok(())
}

#[command]
#[only_in(guilds)]
#[checks(exists_player)]
pub async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let map = ctx.data.read().await;
    map.get::<NightingaleKey>()
        .expect("Set on startup")
        .read()
        .await
        .leave(msg.guild_id.unwrap())
        .await?;
    Ok(())
}

#[command]
#[only_in(guilds)]
#[checks(exists_player)]
async fn pause(ctx: &Context, msg: &Message) -> CommandResult {
    let map = ctx.data.read().await;
    map.get::<NightingaleKey>()
        .expect("Set on startup")
        .read()
        .await
        .get_player_mut(msg.guild_id.unwrap())
        .expect("Check ensures this exists")
        .pause()
        .await?;
    Ok(())
}

#[command]
#[only_in(guilds)]
#[checks(exists_player)]
async fn resume(ctx: &Context, msg: &Message) -> CommandResult {
    let map = ctx.data.read().await;
    map.get::<NightingaleKey>()
        .expect("Set on startup")
        .read()
        .await
        .get_player_mut(msg.guild_id.unwrap())
        .expect("Check ensures this exists")
        .resume()
        .await?;
    Ok(())
}

#[command]
#[only_in(guilds)]
#[checks(exists_player)]
async fn play(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let map = ctx.data.read().await;
    let client = map.get::<NightingaleKey>().expect("Set on startup").read().await;
    let q = args.rest().to_string();

    let url = if URL_REGEX.is_match(&q) {
        q
    } else {
        msg.channel_id.say(ctx, format!("Searching `{q}`")).await?;
        let mut results = client.search(q.clone(), Youtube).await?;

        if results.is_empty() {
            msg.channel_id.say(ctx, format!("No results were found for query: {q}")).await?;
            return Ok(());
        }

        results.remove(0).url
    };

    let track = client.get_player_mut(msg.guild_id.unwrap()).expect("Check ensures this exists")
        .enqueue(Link(url)).await?;

    msg.channel_id.say(ctx, format!("Playing {}", track.title.unwrap())).await?;


    Ok(())
}
