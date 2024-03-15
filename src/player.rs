use std::num::NonZeroU64;
#[cfg(feature = "serenity")]
use serenity::gateway::ShardRunnerMessage;
#[cfg(feature = "serenity")]
use futures::channel::mpsc::UnboundedSender as Sender;
use futures::SinkExt;
use serde_json::json;
#[cfg(feature = "twilight")]
use twilight_gateway::MessageSender;
use typemap_rev::TypeMap;
use crate::error::HttpError;
use crate::model::connection::PartialConnectionInfo;
use crate::model::player::PlayerInfo;
use crate::model::track::Track;
use crate::rest::RestClient;
use crate::source::PlaySource;

/// A player assigned to a guild.
pub struct Player {
    http: RestClient,
    pub(crate) queue: Vec<Track>,
    pub(crate) current: Option<Track>,
    paused: bool,
    volume: u8,
    deaf: bool,
    mute: bool,
    data: TypeMap,
    guild: NonZeroU64,
    pub(crate) channel: Option<NonZeroU64>,
    #[cfg(feature = "serenity")]
    shard: Sender<ShardRunnerMessage>,
    #[cfg(feature = "twilight")]
    shard: MessageSender,
    pub(crate) info: PartialConnectionInfo
}

impl Player {
    #[cfg(feature = "serenity")]
    pub(crate) fn new(http: RestClient, guild: NonZeroU64, shard: Sender<ShardRunnerMessage>) -> Self {
        Self {
            http,
            queue: Vec::new(),
            current: None,
            data: TypeMap::new(),
            guild,
            channel: None,
            paused: false,
            volume: 100,
            deaf: false,
            mute: false,
            shard,
            info: Default::default()
        }
    }

    #[cfg(feature = "twilight")]
    pub(crate) fn new(http: RestClient, guild: NonZeroU64, shard: MessageSender) -> Self {
        Self {
            http,
            queue: Vec::new(),
            current: None,
            data: TypeMap::new(),
            guild,
            channel: None,
            paused: false,
            volume: 100,
            deaf: false,
            mute: false,
            shard,
            info: Default::default()
        }
    }

    /// Returns the inner type map held by the player.
    pub fn data(&self) -> &TypeMap {
        &self.data
    }

    /// Returns the track that is currently being played, if someone.
    pub fn current(&self) -> &Option<Track> {
        &self.current
    }

    /// Returns the queue of the player.
    pub fn queue(&self) -> &Vec<Track> {
        &self.queue
    }

    /// Gets the information held by the server about the player.
    pub async fn info(&self) -> Result<PlayerInfo, HttpError> {
        self.http.player_info(self.guild).await
    }

    /// Enqueues the provided track to be played.
    pub async fn enqueue(&mut self, source: impl PlaySource) -> Result<Track, HttpError> {
        let t = self.http.player_play(self.guild, source, false).await?;

        self.queue.push(t.clone());
        Ok(t)
    }

    /// Pauses the currently playing track and forces the provided one to play at arrival.
    pub async fn force_play(&mut self, source: impl PlaySource) -> Result<Track, HttpError> {
        let t = self.http.player_play(self.guild, source, true).await?;
        self.queue.insert(0, t.clone());
        Ok(t)
    }

    /// Pauses the playback if possible.
    pub async fn pause(&mut self) -> Result<(), HttpError> {
        if self.paused {
            Ok(())
        } else {
            self.http.player_pause(self.guild).await
                .map(|p| {
                    self.paused = true;
                    p
                })
        }
    }

    /// Resumes the playback if possible.
    pub async fn resume(&mut self) -> Result<(), HttpError> {
        if !self.paused {
            Ok(())
        } else {
            self.http.player_resume(self.guild).await
                .map(|p| {
                    self.paused = false;
                    p
                })
        }
    }

    /// Sets a new volume, the default value is 100.
    pub async fn set_volume(&mut self, volume: u8) -> Result<(), HttpError> {
        if self.volume == volume {
            Ok(())
        } else {
            self.http.player_set_volume(self.guild, volume).await
                .map(|r| {
                    self.volume = volume;

                    r
                })
        }
    }

    async fn update(&mut self, channel: Option<NonZeroU64>) {
        let value = json!({
            "op": 4,
            "d": {
                "channel_id": channel.map(|c| c.get()),
                "guild_id": self.guild,
                "self_deaf": self.deaf,
                "self_mute": self.mute,
            }
        });

        #[cfg(feature = "serenity")]
        {
            let _ = self.shard.send(ShardRunnerMessage::Message(value.to_string().into())).await;
        }

        #[cfg(feature = "twilight")]
        {
            let _ = self.shard.send(value.to_string());
        }
    }

    pub async fn set_deaf(&mut self, deaf: bool) {
        self.deaf = deaf;
        self.update(self.channel).await;
    }

    pub async fn set_mute(&mut self, mute: bool) {
        self.mute = mute;
        self.update(self.channel).await;
    }

    pub async fn connect_to(&mut self, channel: impl Into<NonZeroU64>) {
        self.update(Some(channel.into())).await;
    }

    pub async fn disconnect(&mut self) -> Result<(), HttpError> {
        let value = json!({
            "op": 4,
            "d": {
                "channel_id": null,
                "guild_id": self.guild.get(),
                "self_deaf": self.deaf,
                "self_mute": self.mute,
            }
        });

        #[cfg(feature = "serenity")]
        {
            let _ = self.shard.send(ShardRunnerMessage::Message(value.to_string().into())).await;
        }

        #[cfg(feature = "twilight")]
        {
            let _ = self.shard.send(value.to_string());
        }

        self.http.update_player(self.guild, None).await
    }

    pub(crate) async fn update_state(&mut self) -> Result<(), HttpError> {
        if !self.info.complete() {
            return Ok(())
        }

        let info = std::mem::take(&mut self.info).into_info();

        self.http.update_player(self.guild, Some(info)).await
    }
}
