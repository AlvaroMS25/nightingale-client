use std::num::NonZeroU64;
use typemap_rev::TypeMap;
use crate::error::HttpError;
use crate::model::player::PlayerInfo;
use crate::model::track::Track;
use crate::rest::RestClient;
use crate::source::PlaySource;

/// A player assigned to a guild.
pub struct Player {
    http: RestClient,
    queue: Vec<Track>,
    current: Option<Track>,
    paused: bool,
    volume: u8,
    data: TypeMap,
    guild: NonZeroU64
}

impl Player {
    pub(crate) fn new(http: RestClient, guild: NonZeroU64) -> Self {
        Self {
            http,
            queue: Vec::new(),
            current: None,
            data: TypeMap::new(),
            guild,
            paused: false,
            volume: 100
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
}
