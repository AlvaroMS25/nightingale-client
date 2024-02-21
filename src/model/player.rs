use std::num::NonZeroU64;
use serde::Deserialize;
use crate::model::track::Track;

/// Serializable player object returned from the player info route.
#[derive(Deserialize)]
pub struct PlayerInfo {
    pub guild_id: NonZeroU64,
    pub channel_id: Option<NonZeroU64>,
    pub paused: bool,
    pub volume: u8,
    pub currently_playing: Option<Track>,
    pub queue: Vec<Track>
}