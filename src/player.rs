use std::num::NonZeroU64;
use typemap_rev::TypeMap;
use crate::model::track::Track;
use crate::rest::RestClient;

pub struct Player {
    http: RestClient,
    queue: Vec<Track>,
    current: Option<Track>,
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
            guild
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

    pub async fn play() {}
}
