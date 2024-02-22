use std::num::NonZeroU64;

#[derive(Clone, Default)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub password: String,
    pub ssl: bool,
    pub user_id: Option<NonZeroU64>,
    pub shards: Option<u64>,
    pub connection_attempts: u32
}
