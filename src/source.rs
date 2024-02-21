use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::{json, Value};
use crate::model::search::youtube::{YoutubePlaylist, YoutubeTrack};

pub trait SearchRoute {
    fn track(query: String) -> String;
    fn playlist(playlist: String) -> String;
}

pub trait SearchSource: SearchRoute {
    type Track: DeserializeOwned;
    type Playlist: DeserializeOwned;
}

pub struct Youtube;

impl SearchRoute for Youtube {
    fn track(query: String) -> String {
        format!("/youtube/search?query={query}")
    }

    fn playlist(playlist: String) -> String {
        format!("/youtube/playlist?playlist_id={playlist}")
    }
}

impl SearchSource for Youtube {
    type Track = YoutubeTrack;
    type Playlist = YoutubePlaylist;
}

pub trait PlaySource {
    fn value_for(self) -> Value;
}

pub struct Link(pub String);

impl PlaySource for Link {
    fn value_for(self) -> Value {
        json!({
            "type": "link",
            "data": self.0
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
