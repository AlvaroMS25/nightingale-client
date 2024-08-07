use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeezerTrack {
    pub id: usize,
    pub author: String,
    pub length: u64,
    pub title: String,
    pub uri: String,
    pub artwork_url: Option<String>,
    pub isrc: Option<String>
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeezerPlaylist {
    pub id: usize,
    pub title: String,
    pub description: String,
    pub public: bool,
    pub link: String,
    pub picture: String,
    pub track_number: usize,
    pub duration: usize,
    pub creator: String,
    pub tracks: Vec<DeezerTrack>
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeezerAlbum {
    pub id: usize,
    pub title: String,
    pub link: String,
    pub cover: String,
    pub track_number: usize,
    pub duration: usize,
    pub author: String,
    pub tracks: Vec<DeezerTrack>
}
