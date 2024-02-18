use serenity::async_trait;
use crate::model::gateway::{ready::Ready};
use crate::model::gateway::event::{TrackEnd, TrackErrored};
use crate::model::gateway::state::{ConnectionData, DisconnectData};
use crate::model::track::Track;
use crate::player::Player;

#[async_trait]
pub trait EventHandler: Send + Sync {
    async fn on_ready(&self, _ready: Ready) {}
    async fn on_gateway_connect(&self, _info: ConnectionData) {}
    async fn on_gateway_reconnect(&self, _info: ConnectionData) {}
    async fn on_gateway_disconnect(&self, _info: DisconnectData) {}
    async fn on_track_start(&self, _player: &Player, _track: Track) {}
    async fn on_track_end(&self, _player: &Player, _track_end: TrackEnd) {}
    async fn on_track_errored(&self, _player: &Player, _track_errored: TrackErrored) {}
}