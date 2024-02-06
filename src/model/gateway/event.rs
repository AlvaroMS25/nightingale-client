use serde::Deserialize;

use crate::model::track::Track;

/// Track related events received from the gateway.
#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum Event {
    /// A track has started playing,
    TrackStart(Track),
    /// A track had an error while playing or trying to play,
    TrackErrored(TrackErrored),
    /// A track ended playing, either by skipping or naturally finished,
    TrackEnd(TrackEnd)
}

/// Event fired when a track had an error.
#[derive(Debug, Deserialize)]
pub struct TrackErrored {
    /// The error that occurred.
    pub error: String,
    /// The track itself.
    pub track: Track
}

/// Event fired when a track finishes its playback.
#[derive(Debug, Deserialize)]
pub struct TrackEnd {
    /// Whether if the track was stopped manually.
    pub stopped: bool,
    /// The track itself.
    pub track: Track
}
