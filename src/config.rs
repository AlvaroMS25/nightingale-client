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

impl Default for Config {
    fn default() -> Self {
        Self {
            host: String::from("localhost"),
            port: 8081,
            password: String::from("mypassword"),
            ssl: false,
            user_id: unsafe { NonZeroU64::new_unchecked(1) },
            shards: 1,
            connection_attempts: 5
        }
    }
}
