use std::sync::Arc;
use parking_lot::RwLock;
use uuid::Uuid;
use crate::Shared;

pub struct RestClient {
    shared: Arc<Shared>
}

impl RestClient {
    pub(crate) fn new(shared: Arc<Shared>) -> Self {
        Self {
            shared
        }
    }
}
