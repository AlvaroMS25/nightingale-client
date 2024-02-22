use std::num::NonZeroU64;
use typemap_rev::TypeMap;
use crate::error::HttpError;
use crate::model::player::PlayerInfo;
use crate::model::track::Track;
use crate::rest::RestClient;
use crate::source::PlaySource;

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

    pub fn data(&self) -> &TypeMap {
        &self.data
    }

    pub fn current(&self) -> &Option<Track> {
        &self.current
    }

    pub fn queue(&self) -> &Vec<Track> {
        &self.queue
    }

    pub async fn info(&self) -> Result<PlayerInfo, HttpError> {
        self.http.player_info(self.guild).await
    }

    pub async fn enqueue(&mut self, source: impl PlaySource) -> Result<Track, HttpError> {
        let t = self.http.player_play(self.guild, source, false).await?;

        self.queue.push(t.clone());
        Ok(t)
    }

    pub async fn force_play(&mut self, source: impl PlaySource) -> Result<Track, HttpError> {
        let t = self.http.player_play(self.guild, source, true).await?;
        self.queue.insert(0, t.clone());
        Ok(t)
    }

    pub async fn pause(&mut self) -> Result<(), HttpError> {
        if self.paused {
            Ok(())
        } else {
            self.http.player_pause(self.guild).await
                .map(|p| {
                    self.paused = false;
                    p
                })
        }
    }

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
