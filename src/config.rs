use std::num::NonZeroU64;
use std::time::Duration;

#[derive(Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub password: String,
    pub ssl: bool,
    pub user_id: Option<NonZeroU64>,
    pub connection_attempts: u32
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: String::from("localhost"),
            port: 8081,
            password: String::from("mypassword"),
            ssl: false,
            user_id: None,
            connection_attempts: 5
        }
    }
}

pub struct SessionConfig {
    enable_resume: bool,
    reconnect_time: Duration
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            enable_resume: true,
            reconnect_time: Duration::from_secs(60)
        }
    }
}
