use typemap_rev::TypeMap;
use crate::model::track::Track;

pub struct Player {
    queue: Vec<Track>,
    current: Option<Track>,
    data: TypeMap
}

impl Player {
    pub(crate) fn new() -> Self {
        Self {
            queue: Vec::new(),
            current: None,
            data: TypeMap::new()
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
}
