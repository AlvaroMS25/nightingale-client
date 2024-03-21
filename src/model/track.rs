use std::time::Duration;

use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Track {
    pub track: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub channel: Option<String>,
    #[serde(deserialize_with = "duration_from_millis")]
    pub duration: Option<Duration>,
    pub source_url: Option<String>,
    pub title: Option<String>,
    pub thumbnail: Option<String>
}

fn duration_from_millis<'de, D>(deserializer: D) -> Result<Option<Duration>, D::Error>
where
    D: Deserializer<'de>
{
    Ok(<Option<u128> as Deserialize>::deserialize(deserializer)?
        .map(|millis| Duration::from_millis(millis as _)))
}
