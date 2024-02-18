use std::num::NonZeroU64;

#[derive(Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub password: String,
    pub ssl: bool,
    pub user_id: NonZeroU64,
    pub shards: u64,
    pub connection_attempts: u32
}