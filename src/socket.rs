use std::{pin::Pin, task::{Context, Poll}};

use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::{handshake::client::Request, Error, Message}, MaybeTlsStream, WebSocketStream};
use futures::{ready, Stream, StreamExt};
use tracing::warn;

use crate::{error::SocketError, model::gateway::IncomingPayload};

/// A websocket client to te gateway.
pub(crate) struct Socket {
    stream: Option<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    password: String
}

impl Socket {
    pub fn new(password: String) -> Self {
        Self {
            stream: None,
            password
        }
    }

    pub fn is_connected(&self) -> bool {
        self.stream.is_some()
    }

    pub async fn connect(&mut self, url: String) -> Result<(), SocketError>{
        let req = Request::builder()
            .method("GET")
            .uri(url)
            .header("Authorization", &self.password)
            .body(())
            .unwrap();

        let (connection, _) = connect_async(req).await.map_err(SocketError::Tungstenite)?;

        self.stream = Some(connection);

        Ok(())
    }
}

impl Stream for Socket {
    type Item = Result<IncomingPayload, SocketError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        let Some(socket) = this.stream.as_mut() else { return Poll::Ready(None) };

        let msg = match ready!(Pin::new(socket).poll_next(cx)) {
            None => return Poll::Ready(None),
            Some(Err(e)) => {
                warn!("Disconnected from server, error: {e}");
                this.stream = None;
                return Poll::Ready(Some(Err(From::from(e))));
            },
            Some(Ok(msg)) => msg
        };

        let data = match msg {
            Message::Text(t) => t,
            _ => return Poll::Pending
        };

        Poll::Ready(Some(serde_json::from_str(&data).map_err(From::from)))
    }
}
