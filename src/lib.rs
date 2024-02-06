pub mod model;
pub mod config;
mod error;
mod socket;

use reqwest::Client;
use socket::Socket;
use config::Config;

pub struct NightingaleClient {
    socket: Socket,
    http: Client,
    config: Config
}

impl NightingaleClient {
    pub fn new(config: Config) -> Self {
        todo!()
    }
}
