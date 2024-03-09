use std::error::Error;
use lazy_static::lazy_static;
use nightingale_client::source::{Link, Youtube};
use regex::Regex;
use tracing::info;
use twilight_interactions::command::{CommandModel, CreateCommand, CreateOption};
use twilight_interactions::command::internal::{CommandOptionData, CreateOptionData};
use twilight_interactions::error::ParseOptionErrorType;
use twilight_model::application::command::{CommandOption, CommandOptionType, CommandOptionValue};
use twilight_model::application::interaction::{Interaction, InteractionData};
use twilight_model::application::interaction::application_command::CommandInteractionDataResolved;
use twilight_model::http::interaction::{InteractionResponse, InteractionResponseType};
use crate::{ArcShared, Shared};

lazy_static! {
    static ref URL_REGEX: Regex = Regex::new(r"https?://(www\.)?[-a-zA-Z0-9@:%._\+~#=]{2,256}\.[a-z]{2,4}\b([-a-zA-Z0-9@:%_\+.~#?&//=]*)").unwrap();
}

/// Plays from the specified source or query
#[derive(CommandModel, CreateCommand, Debug)]
#[command(name = "play")]
pub struct Play {
    /// Query or link to play from
    source: String
}

/// Joins the specified channel
#[derive(CommandModel, CreateCommand, Debug)]
#[command(name = "join")]
pub struct Join;

/// Leaves the channel
#[derive(CommandModel, CreateCommand, Debug)]
#[command(name = "leave")]
pub struct Leave;

/// Pauses playback
#[derive(CommandModel, CreateCommand, Debug)]
#[command(name = "pause")]
pub struct Pause;

/// Resumes playback
#[derive(CommandModel, CreateCommand, Debug)]
#[command(name = "resume")]
pub struct Resume;

/// Changes the player volume
#[derive(CommandModel, CreateCommand, Debug)]
#[command(name = "set_volume")]
pub struct SetVolume {
    /// The new volume
    volume: U8
}

#[derive(Debug)]
struct U8(u8);

impl CreateOption for U8 {
    fn create_option(data: CreateOptionData) -> CommandOption {
        let mut option = data.into_option(CommandOptionType::Integer);

        option.min_value = Some(CommandOptionValue::Integer(u8::MIN as _));
        option.max_value = Some(CommandOptionValue::Integer(u8::MAX as _));

        option
    }
}

impl twilight_interactions::command::CommandOption for U8 {
    fn from_option(
        value: twilight_model::application::interaction::application_command::CommandOptionValue,
        _: CommandOptionData,
        _: Option<&CommandInteractionDataResolved>
    ) -> Result<Self, ParseOptionErrorType> {
        use twilight_model::application::interaction::application_command::CommandOptionValue as V;

        let V::Integer(i) = &value else {
            return Err(ParseOptionErrorType::InvalidType(value.kind()))
        };

        if i < &(u8::MIN as _) || i > &(u8::MAX as _) {
            Err(ParseOptionErrorType::IntegerOutOfRange(*i))
        } else {
            Ok(U8(*i as _))
        }
    }
}

pub async fn execute(
    shared: ArcShared,
    interaction: Interaction
) -> Result<(), Box<dyn Error>> {
    let Some(InteractionData::ApplicationCommand(cmd)) = &interaction.data else {
        return Ok(());
    };

    match cmd.name.as_str() {
        "play" => {
            Play::from_interaction((*cmd.clone()).into())?.run(&shared, interaction)
                .await?;
        },
        "join" => {
            Join.run(&shared, interaction).await?;
        },
        "leave" => {
            Leave.run(&shared, interaction).await?;
        }
        "pause" => {
            Pause.run(&shared, interaction).await?;
        },
        "resume" => {
            Resume.run(&shared, interaction).await?;
        },
        "set_volume" => {
            SetVolume::from_interaction((*cmd.clone()).into())?.run(&shared, interaction)
                .await?;
        }
        _ => ()
    }

    Ok(())
}

async fn defer(shared: &Shared, interaction: &Interaction) -> Result<(), Box<dyn Error>> {
    shared.http.interaction(shared.app_id)
        .create_response(
            interaction.id,
            &interaction.token,
            &InteractionResponse {
                kind: InteractionResponseType::DeferredChannelMessageWithSource,
                data: None,
            },
        )
        .await
        .map(|_| ())
        .map_err(From::from)
}

async fn send_text(
    shared: &Shared,
    interaction: &Interaction,
    content: String
) -> Result<(), Box<dyn Error>> {
    shared.http.interaction(shared.app_id)
        .update_response(&interaction.token)
        .content(Some(&content))?
        .await
        .map(|_| ())
        .map_err(From::from)
}

async fn player_available(shared: &Shared, interaction: &Interaction) -> bool {
    let p = shared.nightingale.read().await
        .get_player(interaction.guild_id.unwrap())
        .is_some();
    info!("Player available?: {p}");

    p
}

impl Play {
    async fn run(self, shared: &Shared, interaction: Interaction) -> Result<(), Box<dyn Error>> {
        if !player_available(shared, &interaction).await {
            return Ok(())
        }

        defer(shared, &interaction).await?;

        let client = shared.nightingale.read().await;

        let src = if URL_REGEX.is_match(&self.source) {
            self.source
        } else {
            let mut results = client.search(self.source.clone(), Youtube).await?;

            if results.is_empty() {
                send_text(
                    shared,
                    &interaction,
                    format!("No results were found for query: {}", self.source)
                ).await?;
                return Ok(());
            }

            results.remove(0).url
        };

        let track = client.get_player_mut(interaction.guild_id.unwrap())
            .unwrap()
            .enqueue(Link(src))
            .await?;

        send_text(shared, &interaction, format!("Playing {}", track.title.unwrap()).to_string())
            .await
    }
}

impl Join {
    async fn run(self, shared: &Shared, interaction: Interaction) -> Result<(), Box<dyn Error>> {
        let vs = shared.cache.voice_state(
            interaction.author_id().unwrap(),
            interaction.guild_id.unwrap()
        ).unwrap();

        defer(shared, &interaction).await?;

        shared.nightingale.read().await
            .join(vs.guild_id(), vs.channel_id())
            .await?;


        send_text(shared, &interaction, "Joined channel!".to_string()).await
    }
}

impl Leave {
    async fn run(self, shared: &Shared, interaction: Interaction) -> Result<(), Box<dyn Error>> {
        if !player_available(shared, &interaction).await {
            return Ok(())
        }

        defer(shared, &interaction).await?;

        shared.nightingale.read()
            .await
            .leave(interaction.guild_id.unwrap())
            .await?;

        send_text(shared, &interaction, "Left channel!".to_string()).await
    }
}

impl Pause {
    async fn run(self, shared: &Shared, interaction: Interaction) -> Result<(), Box<dyn Error>> {
        if !player_available(shared, &interaction).await {
            return Ok(())
        }

        defer(shared, &interaction).await?;

        shared.nightingale.read()
            .await
            .get_player_mut(interaction.guild_id.unwrap())
            .unwrap()
            .pause()
            .await?;

        send_text(shared, &interaction, "Player paused!".to_string()).await
    }
}

impl Resume {
    async fn run(self, shared: &Shared, interaction: Interaction) -> Result<(), Box<dyn Error>> {
        if !player_available(shared, &interaction).await {
            return Ok(())
        }

        defer(shared, &interaction).await?;

        shared.nightingale.read()
            .await
            .get_player_mut(interaction.guild_id.unwrap())
            .unwrap()
            .resume()
            .await?;

        send_text(shared, &interaction, "Player resumed!".to_string()).await
    }
}

impl SetVolume {
    async fn run(self, shared: &Shared, interaction: Interaction) -> Result<(), Box<dyn Error>> {
        if !player_available(shared, &interaction).await {
            return Ok(())
        }

        defer(shared, &interaction).await?;

        shared.nightingale.read()
            .await
            .get_player_mut(interaction.guild_id.unwrap())
            .unwrap()
            .set_volume(self.volume.0)
            .await?;

        send_text(shared, &interaction, format!("Set volume to {}", self.volume.0)).await
    }
}
