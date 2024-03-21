use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::{json, Value};
use crate::model::search::youtube::{YoutubePlaylist, YoutubeTrack};
use crate::model::track::Track;

/// Represents the routes of the different search sources.
pub trait SearchRoute {
    /// Returns the route to search `query` with this source.
    fn track(query: String) -> String;
    /// Returns the route to search `playlist` with this route.
    fn playlist(playlist: String) -> String;
}

/// Represents a source that can be used to search and query tracks.
pub trait SearchSource: SearchRoute {
    /// The track type returned from search calls.
    type Track: DeserializeOwned;
    /// The playlist returned from playlist calls.
    type Playlist: DeserializeOwned;
}

/// Youtube source.
///
/// This is the only search source supported at the moment.
pub struct Youtube;

impl SearchRoute for Youtube {
    fn track(query: String) -> String {
        format!("/youtube/search?query={}", urlencoding::encode(&query))
    }

    fn playlist(playlist: String) -> String {
        format!("/youtube/playlist?playlist={}", urlencoding::encode(&playlist))
    }
}

impl SearchSource for Youtube {
    type Track = YoutubeTrack;
    type Playlist = YoutubePlaylist;
}

/// Represents sources that can actually be played from.
pub trait PlaySource {
    /// Returns the json payload to play from the specified source. This method
    /// must return only the part that would be inside the `source` field.
    fn value_for(self) -> Value;
}

pub struct Link(pub String);

impl PlaySource for Link {
    fn value_for(self) -> Value {
        json!({
            "type": "link",
            "data": {
                "link": self.0
            }
        })
    }
}

pub struct Http(pub String);
impl PlaySource for Http {
    fn value_for(self) -> Value {
        json!({
            "type": "http",
            "data": {
                "link": self.0
            }
        })
    }
}

pub struct HttpWithTrack(pub String, pub Track);

impl PlaySource for HttpWithTrack {
    fn value_for(self) -> Value {
        let serialized = serde_json::to_value(&self.1).expect("Shouldn't fail");
        json!({
            "type": "http",
            "data": {
                "link": self.0,
                "track": serialized
            }
        })
    }
}

pub struct ForceYtDlp(pub String);

impl PlaySource for ForceYtDlp {
    fn value_for(self) -> Value {
        json!({
            "type": "http",
            "data": {
                "link": self.0,
                "force_ytdlp": true
            }
        })
    }
}

pub struct Bytes(pub Vec<u8>);

impl PlaySource for Bytes {
    fn value_for(self) -> Value {
        json!({
            "type": "bytes",
            "data": self.0
        })
    }
}
