use std::sync::Arc;
use parking_lot::RwLock;
use uuid::Uuid;

pub struct RestClient {
    session: Arc<RwLock<Uuid>>
}

impl RestClient {
    pub(crate) fn new(session: Arc<RwLock<Uuid>>) -> Self {
        Self {
            session
        }
    }
}
