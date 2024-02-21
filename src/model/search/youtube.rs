use serde::Deserialize;

#[derive(Deserialize)]
pub struct YoutubePlaylist {
    /// Name of the playlist.
    pub name: String,
    /// Tracks of the playlist.
    pub tracks: Vec<YoutubeTrack>
}

#[derive(Deserialize)]
pub struct YoutubeTrack {
    /// Title of the track.
    pub title: String,
    /// Author of the track if available.
    pub author: Option<String>,
    /// Length of the track in milliseconds.
    pub length: u128,
    /// Id of the video.
    pub video_id: String,
    /// Whether if the video is a stream.
    pub is_stream: bool,
    /// The url of the video.
    pub url: String,
    /// The thumbnail of the video.
    pub thumbnail: String
}
