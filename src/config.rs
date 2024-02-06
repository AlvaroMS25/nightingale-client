use std::num::NonZeroU64;

pub struct Config {
    pub host: String,
    pub port: u16,
    pub password: String,
    pub ssl: bool,
    pub user_id: NonZeroU64,
    pub shards: u64
}