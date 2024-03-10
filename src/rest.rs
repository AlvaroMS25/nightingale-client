use std::fmt::format;
use std::num::NonZeroU64;
use std::sync::Arc;
use parking_lot::RwLock;
use reqwest::{Error, Client, Response};
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};
use serde::de::DeserializeOwned;
use serde_json::json;
use uuid::Uuid;
use crate::error::{HttpError, StatusCodeError};
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
        let pass = shared.config.read().password.clone();

        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&pass).expect("Invalid password"));

        Self {
            shared,
            http: Client::builder().default_headers(headers).build().unwrap()
        }
    }

    fn base_api_route(&self) -> String {
        let config = self.shared.config.read();
        let schema = if config.ssl {
            "https://"
        } else {
            "http://"
        };

        format!("{schema}{}:{}/api/v1", config.host, config.port)
    }

    fn session(&self) -> Uuid {
        *self.shared.session.read()
    }

    /// Searches from the specified source, returning a vector of results.
    pub async fn search<S>(&self, query: String, source: S) -> Result<Vec<S::Track>, HttpError>
    where
        S: SearchSource
    {
        let _ = source;
        deserialize_json::<Vec<S::Track>>(
            self.http.get(format!("{}/search{}", self.base_api_route(), S::track(query)))
                .send()
                .await?
        ).await
    }

    /// Queries the playlist items, returning them and the playlist name.
    pub async fn playlist<S>(&self, playlist: String, source: S) -> Result<S::Playlist, HttpError>
    where
        S: SearchSource
    {
        let _ = source;
        deserialize_json::<S::Playlist>(
            self.http.get(format!("{}/search{}", self.base_api_route(), S::playlist(playlist)))
                .send()
                .await?
        ).await
    }

    /// Returns information about the server. If `current_session` is set to `true`, then the playback
    /// field will only represent the current session players.
    pub async fn server_info(&self, current_session: bool) -> Result<Info, HttpError> {
        let mut url = format!("{}/info", self.base_api_route());

        if current_session {
            url.push('/');
            url.push_str(self.session().to_string().as_str());
        }

        let res = self.http.get(url)
            .send()
            .await?;

        if res.status().is_success() {
            res.json().await.map_err(From::from)
        } else {
            Err(HttpError::UnexpectedStatus(StatusCodeError(res.status())))
        }
    }

    pub(crate) async fn connect(&self, guild: NonZeroU64, channel: NonZeroU64) -> Result<(), HttpError> {
        let session = self.session();

        let url = format!(
            "{}/{session}/players/{guild}/connect?channel_id={channel}",
            self.base_api_route()
        );
        let res = self.http.put(url)
            .send()
            .await?;

        if res.status().is_success() {
            Ok(())
        } else {
            Err(res.json::<ErrorResponse>()
                .await
                .map(HttpError::ErrorMessage)
                .unwrap_or_else(From::from))
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
            Err(res.json::<ErrorResponse>()
                .await
                .map(HttpError::ErrorMessage)
                .unwrap_or_else(From::from))
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

        deserialize_json(self.http.post(url).json(&body).send().await?).await
    }

    pub(crate) async fn player_pause(&self, guild: NonZeroU64) -> Result<(), HttpError>
    {
        let session = self.session();
        let url = format!("{}/{session}/players/{guild}/pause", self.base_api_route());
        let res = self.http.patch(url).send().await?;

        if res.status().is_success() {
            Ok(())
        } else {
            Err(res.json::<ErrorResponse>()
                .await
                .map(HttpError::ErrorMessage)
                .unwrap_or_else(From::from))
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
            Err(res.json::<ErrorResponse>()
                .await
                .map(HttpError::ErrorMessage)
                .unwrap_or_else(From::from))
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
            Err(res.json::<ErrorResponse>()
                .await
                .map(HttpError::ErrorMessage)
                .unwrap_or_else(From::from))
        }
    }

}

async fn deserialize_json<M: DeserializeOwned>(response: Response) -> Result<M, HttpError> {
    if response.status().is_success() {
        Ok(response.json().await?)
    } else {
        Err(response.json::<ErrorResponse>().await
            .map(HttpError::ErrorMessage)
            .unwrap_or_else(From::from))
    }
}
