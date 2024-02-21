use std::num::NonZeroU64;

#[derive(Clone, Default)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub password: String,
    pub ssl: bool,
    pub(crate) user_id: Option<NonZeroU64>,
    pub(crate) shards: Option<u64>,
    pub connection_attempts: u32
}