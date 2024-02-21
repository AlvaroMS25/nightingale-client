use std::num::NonZeroU64;
use std::sync::Arc;
use parking_lot::RwLock;
use reqwest::{Error, Client, Response};
use serde::de::DeserializeOwned;
use serde_json::json;
use uuid::Uuid;
use crate::error::HttpError;
use crate::model::error::ErrorResponse;
use crate::model::info::Info;
use crate::model::player::PlayerInfo;
use crate::model::track::Track;
use crate::Shared;
use crate::source::{PlaySource, SearchSource};

#[derive(Clone)]
pub struct RestClient {
    shared: Arc<Shared>,
    http: Client
}

impl RestClient {
    pub(crate) fn new(shared: Arc<Shared>) -> Self {
        Self {
            shared,
            http: Client::new()
        }
    }

    fn base_route(&self) -> String {
        let config = self.shared.config.read();
        let schema = if config.ssl {
            "https://"
        } else {
            "http://"
        };

        format!("{schema}{}:{}", config.host, config.port)
    }

    fn base_api_route(&self) -> String {
        format!("{}/api/v1", self.base_route())
    }

    fn session(&self) -> Uuid {
        *self.shared.session.read()
    }

    pub async fn search<S>(&self, query: String, source: S) -> Result<S::Track, HttpError>
    where
        S: SearchSource
    {
        let _ = source;
        deserialize_json(
            self.http.get(format!("{}{}", self.base_route(), S::track(query)))
                .send()
                .await?
        ).await
    }

    pub async fn playlist<S>(&self, playlist: String, source: S) -> Result<S::Playlist, HttpError>
    where
        S: SearchSource
    {
        let _ = source;
        deserialize_json(
            self.http.get(format!("{}/search{}", self.base_route(), S::playlist(playlist)))
                .send()
                .await?
        ).await
    }

    pub async fn server_info(&self) -> Result<Info, HttpError> {
        let res = self.http.get(format!("{}/info", self.base_route()))
            .send()
            .await?;

        if res.status().is_success() {
            res.json().await.map_err(From::from)
        } else {
            Err(HttpError::UnexpectedStatus(res.status()))
        }
    }

    pub(crate) async fn connect(&self, session: Uuid, guild: NonZeroU64) -> Result<(), HttpError> {
        let url = format!("{}/{session}/players/{guild}/connect", self.base_api_route());
        let res = self.http.put(url)
            .send()
            .await?;

        if res.status().is_success() {
            Ok(())
        } else {
            res.json::<ErrorResponse>()
                .await
                .map_err(From::from)
        }
    }

    pub(crate) async fn disconnect(&self, guild: NonZeroU64) -> Result<(), HttpError>
    {
        let session = self.session();
        let url = format!("{}/{session}/players/{guild}/disconnect", self.base_api_route());
        let res = self.http.delete(url)
            .send()
            .await?;

        if res.status().is_success() {
            Ok(())
        } else {
            res.json::<ErrorResponse>()
                .await
                .map_err(From::from)
        }
    }

    pub(crate) async fn player_info(&self, guild: NonZeroU64) -> Result<PlayerInfo, HttpError>
    {
        let session = self.session();
        let url = format!("{}/{session}/players/{guild}/info", self.base_api_route());

        deserialize_json(self.http.get(url).send().await?).await
    }

    pub(crate) async fn player_play<S>(
        &self,
        guild: NonZeroU64,
        source: S,
        force: bool
    ) -> Result<Track, HttpError>
    where
        S: PlaySource
    {
        let body = json!({
            "force_play": force,
            "source": source.value_for()
        });
        let session = self.session();

        let url = format!("{}/{session}/players/{guild}/play", self.base_api_route());

        deserialize_json(self.http.post(url).json(&body).send().await?)
    }

    pub(crate) async fn player_pause(&self, guild: NonZeroU64) -> Result<(), HttpError>
    {
        let session = self.session();
        let url = format!("{}/{session}/players/{guild}/pause", self.base_api_route());
        let res = self.http.patch(url).send().await?;

        if res.status().is_success() {
            Ok(())
        } else {
            res.json::<ErrorResponse>()
                .await
                .map_err(From::from)
        }
    }

    pub(crate) async fn player_resume(&self, guild: NonZeroU64) -> Result<(), HttpError>
    {
        let session = self.session();
        let url = format!("{}/{session}/players/{guild}/resume", self.base_api_route());
        let res = self.http.patch(url).send().await?;

        if res.status().is_success() {
            Ok(())
        } else {
            res.json::<ErrorResponse>()
                .await
                .map_err(From::from)
        }
    }

    pub(crate) async fn player_set_volume(
        &self,
        guild: NonZeroU64,
        volume: u8
    ) -> Result<(), HttpError>
    {
        let session = self.session();
        let url = format!("{}/{session}/players/{guild}/set_volume/{volume}", self.base_api_route());
        let res = self.http.patch(url).send().await?;

        if res.status().is_success() {
            Ok(())
        } else {
            res.json::<ErrorResponse>()
                .await
                .map_err(From::from)
        }
    }

}

async fn deserialize_json<M: DeserializeOwned>(response: Response) -> Result<M, HttpError> {
    if response.status().is_success() {
        Ok(response.json().await?)
    } else {
        Err(response.json::<ErrorResponse>().await
            .map(HttpError::ErrorMessage)
            .unwrap_or_else(HttpError::Reqwest))
    }
}
