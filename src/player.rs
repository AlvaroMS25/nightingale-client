use std::any::Any;
use std::collections::VecDeque;
use std::num::NonZeroU64;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};
use parking_lot::Mutex;
use crate::error::HttpError;
use crate::model::connection::PartialConnectionInfo;
use crate::model::player::PlayerInfo;
use crate::model::track::Track;
use crate::reflock::{Read, RefLock};
use crate::rest::RestClient;
use crate::source::PlaySource;

/// A player assigned to a guild.
pub struct Player {
    pub(crate) http: RestClient,
    pub(crate) queue: RefLock<VecDeque<Track>>,
    pub(crate) current: RefLock<Option<Track>>,
    pub(crate) paused: AtomicBool,
    pub(crate) volume: AtomicU16,
    pub(crate) data: Option<Arc<dyn Any + Send + Sync + 'static>>,
    pub(crate) guild: NonZeroU64,
    pub(crate) partial: Mutex<PartialConnectionInfo>
}

impl Player {
    pub(crate) fn new(http: RestClient, guild: NonZeroU64) -> Self {
        Self {
            http,
            queue: Default::default(),
            current: Default::default(),
            data: None,
            guild,
            paused: AtomicBool::new(false),
            volume: AtomicU16::new(100),
            partial: Default::default(),
        }
    }
    pub(crate) fn new_with_data<T>(http: RestClient, guild: NonZeroU64, data: T) -> Self
    where
        T: Any + Send + Sync + 'static
    {
        Self {
            http,
            queue: Default::default(),
            current: Default::default(),
            data: Some(Arc::new(data) as Arc<dyn Any + Send + Sync + 'static>),
            guild,
            paused: AtomicBool::new(false),
            volume: AtomicU16::new(100),
            partial: Default::default(),
        }
    }

    /// Returns the inner type map held by the player.
    pub fn data<T: Send + Sync + 'static>(&self) -> Option<&Arc<T>> {
        self.data.as_ref().map(|d| d.downcast_ref()).flatten()
    }

    /// Returns the track that is currently being played, if someone.
    pub fn current(&self) -> Read<Option<Track>> {
        self.current.read()
    }

    /// Returns the queue of the player.
    pub fn queue(&self) -> Read<VecDeque<Track>> {
        self.queue.read()
    }

    pub fn volume(&self) -> u16 {
        self.volume.load(Ordering::Relaxed)
    }

    pub fn paused(&self) -> bool {
        self.paused.load(Ordering::Relaxed)
    }

    /// Gets the information held by the server about the player.
    pub async fn info(&self) -> Result<PlayerInfo, HttpError> {
        self.http.player_info(self.guild).await
    }

    /// Enqueues the provided track to be played.
    pub async fn enqueue(&self, source: impl PlaySource) -> Result<Track, HttpError> {
        let t = self.http.player_play(self.guild, source, false).await?;

        self.queue.write().push_back(t.clone());
        Ok(t)
    }

    /// Pauses the currently playing track and forces the provided one to play at arrival.
    pub async fn force_play(&self, source: impl PlaySource) -> Result<Track, HttpError> {
        let t = self.http.player_play(self.guild, source, true).await?;
        self.queue.write().insert(0, t.clone());
        Ok(t)
    }

    /// Pauses the playback if possible.
    pub async fn pause(&self) -> Result<(), HttpError> {
        if self.paused() {
            Ok(())
        } else {
            self.http.player_pause(self.guild).await
                .map(|p| {
                    self.paused.store(true, Ordering::Relaxed);
                    p
                })
        }
    }

    /// Resumes the playback if possible.
    pub async fn resume(&self) -> Result<(), HttpError> {
        if !self.paused() {
            Ok(())
        } else {
            self.http.player_resume(self.guild).await
                .map(|p| {
                    self.paused.store(false, Ordering::Relaxed);
                    p
                })
        }
    }

    pub async fn skip(&self) -> Result<Option<Track>, HttpError> {
        if self.queue().is_empty() {
            return Ok(None);
        }

        self.queue.write().pop_front();
        self.http.player_skip(self.guild).await
            .map(Some)
    }

    /// Sets a new volume, the default value is 100.
    pub async fn set_volume(&self, volume: u16) -> Result<(), HttpError> {
        if self.volume() == volume {
            Ok(())
        } else {
            self.http.player_set_volume(self.guild, volume).await
                .map(|r| {
                    self.volume.store(volume, Ordering::Relaxed);

                    r
                })
        }
    }
}
