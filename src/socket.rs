use std::{pin::Pin, task::{Context, Poll}};
use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use parking_lot::RwLock;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::{handshake::client::Request, Error, Message}, MaybeTlsStream, WebSocketStream};
use futures::{ready, SinkExt, Stream, StreamExt};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender, UnboundedReceiver};
use tracing::{error, info, warn};
use uuid::Uuid;
use futures::channel::mpsc::UnboundedSender as Sender;
use tokio_tungstenite::tungstenite::protocol::CloseFrame;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use crate::{error::SocketError, model::gateway::IncomingPayload, PlayerManager, Shared};

use crate::config::Config;
use crate::model::gateway::state::UpdateState;
use crate::msg::{FromSocketMessage, ToSocketMessage};

#[cfg(feature = "serenity")]
use crate::events::EventHandler;
#[cfg(feature = "serenity")]
use crate::model::gateway::event::Event;
#[cfg(feature = "serenity")]
use serenity::gateway::ShardRunnerMessage;

pub struct SocketHandle {
    pub sender: UnboundedSender<ToSocketMessage>,
    pub receiver: UnboundedReceiver<FromSocketMessage>
}

/// A websocket client to te gateway.
pub(crate) struct Socket {
    stream: Option<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    shared: Arc<Shared>,
    players: Arc<PlayerManager>,
    sender: UnboundedSender<FromSocketMessage>,
    #[cfg(feature = "serenity")]
    events: Arc<dyn EventHandler + 'static>,
    #[cfg(feature = "serenity")]
    shards: HashMap<u32, Sender<ShardRunnerMessage>>
}

impl Socket {
    #[cfg(feature = "serenity")]
    pub fn new(
        shared: Arc<Shared>,
        players: Arc<PlayerManager>,
        event_handler: Arc<dyn EventHandler + 'static>
    ) -> SocketHandle {
        let (tx, rx) = unbounded_channel();
        let (from_tx, from_rx) = unbounded_channel();
        let this = Self {
            stream: None,
            shared,
            players,
            sender: from_tx,
            events: event_handler,
            shards: HashMap::new()
        };

        tokio::spawn(async move {
            this.run(rx).await;
        });

        SocketHandle {
            sender: tx,
            receiver: from_rx
        }
    }

    async fn run(mut self, mut receiver: UnboundedReceiver<ToSocketMessage>) {
        loop {
            tokio::select! {
                biased;
                msg = receiver.recv() => {
                    let Some(msg) = msg else { continue; };

                    if let ToSocketMessage::Kill = &msg {
                        return;
                    }

                    self.handle_msg(msg).await
                },
                Some(payload) = self.next() => self.handle_payload(payload).await
            }
        }
    }

    fn connect_uri(&self) -> String {
        let config = self.shared.config.read();
        let proto = if config.ssl {
            "wss://"
        } else {
            "ws://"
        };

        format!("{}{}:{}/ws", proto, config.host, config.port)
    }

    fn sender_send(&mut self, msg: FromSocketMessage) {
        let _ = self.sender.send(msg);
    }

    async fn handle_msg(&mut self, msg: ToSocketMessage) {
        match msg {
            ToSocketMessage::Connect | ToSocketMessage::Reconnect => {
                self.try_disconnect().await;

                let url = {
                    let config = self.shared.config.read();

                    format!(
                        "{}?shards={}&user_id={}",
                        self.connect_uri(),
                        config.shards.unwrap(),
                        config.user_id.unwrap()
                    )
                };


                self.try_connect(url).await;
            }
            ToSocketMessage::Disconnect => self.try_disconnect().await,
            ToSocketMessage::Resume => {
                self.try_disconnect().await;
                let session = *self.shared.session.read();

                assert_ne!(session, Uuid::nil());

                let url = format!("{}/resume/{}", self.connect_uri(), session);

                info!("Trying to resume session");

                self.try_connect(url).await;
            },
            ToSocketMessage::Send(payload) => {
                let Ok(serialized) = serde_json::to_string(&payload) else {
                    error!("Failed to serialize payload");
                    return;
                };

                if let Some(socket) = self.stream.as_mut() {
                    let _ = socket.send(Message::Text(serialized)).await;
                }
            },
            #[cfg(feature = "serenity")]
            ToSocketMessage::RegisterShard(id, shard) => {
                self.shards.insert(id, shard);
            },
            #[cfg(feature = "serenity")]
            ToSocketMessage::DeregisterShard(id) => {
                self.shards.remove(&id);
            }
            _ => ()
        }
    }

    async fn try_disconnect(&mut self) {
        let Some(mut conn) = self.stream.take() else { return; };

        let _ = conn.close(Some(CloseFrame {
            code: CloseCode::Normal,
            reason: "".into()
        })).await;

        self.sender_send(FromSocketMessage::Disconnected);
    }

    async fn try_connect(&mut self, url: String) {
        let attempts = self.shared.config.read().connection_attempts;
        for i in 1..=attempts {
            match self.connect(&url).await {
                Ok(_) => {
                    info!("Connected to nightingale server successfully!");
                    self.sender_send(FromSocketMessage::ConnectedSuccessfully);
                    return;
                },
                Err(error) => {
                    warn!(
                        "Failed to connect to nightingale server [Attempt {}/{}]",
                        i,
                        attempts
                    );

                    if i == attempts {
                        self.sender_send(FromSocketMessage::FailedToConnect(error));
                    }
                }
            }
        }

        error!("Failed to connect to nightingale server after {} attempts", attempts);
    }

    async fn handle_payload(&mut self, incoming: Result<IncomingPayload, SocketError>) {
        match incoming {
            Ok(payload) => {
                #[cfg(feature = "serenity")]
                self.handle_payload_inner_serenity(payload).await;
            },
            Err(error) => match error {
                SocketError::Deserialize(e) => {
                    error!("Failed to deserialize payload: {e:?}");
                },
                SocketError::Tungstenite(e) => {
                    error!("Disconnected from server, error: {e}");
                    self.stream = None;
                    self.sender_send(FromSocketMessage::Disconnected);
                }
            }
        }
    }

    #[cfg(feature = "serenity")]
    async fn handle_payload_inner_serenity(&mut self, payload: IncomingPayload) {
        let events = Arc::clone(&self.events);
        match payload {
            IncomingPayload::Ready(r) => {
                *self.shared.session.write() = r.session;

                tokio::spawn(async move {
                    events.on_ready(r).await;
                });
            },
            IncomingPayload::UpdateState(state) => match state {
                UpdateState::ConnectGateway(data) => {
                    tokio::spawn(async move {
                        events.on_gateway_connect(data).await;
                    });
                },
                UpdateState::ReconnectGateway(data) => {
                    tokio::spawn(async move {
                        events.on_gateway_reconnect(data).await;
                    });
                },
                UpdateState::DisconnectGateway(data) => {
                    tokio::spawn(async move {
                        events.on_gateway_disconnect(data).await;
                    });
                }
            },
            IncomingPayload::Forward(forward) => {
                let Some(shard) = self.shards.get(&(forward.shard as u32)) else {
                    error!("Shard {} not found", forward.shard);
                    return;
                };

                let Ok(payload) = serde_json::to_string(&forward.payload) else {
                    error!("Failed to serialize forward payload");
                    return;
                };

                shard.unbounded_send(ShardRunnerMessage::Message(Message::Text(payload))).unwrap()
            }
            IncomingPayload::Event { guild_id, event } => {
                let players = Arc::clone(&self.players);

                tokio::spawn(async move {
                    let player = players.get_or_insert(guild_id);

                    match event {
                        Event::TrackStart(t) => events.on_track_start(&*player, t).await,
                        Event::TrackEnd(t) => events.on_track_end(&*player, t).await,
                        Event::TrackErrored(t) => events.on_track_errored(&*player, t).await
                    }
                });
            }
        }
    }

    async fn connect(&mut self, url: &str) -> Result<(), Error>{
        let req = Request::builder()
            .method("GET")
            .uri(url)
            .header("Authorization", &*self.shared.config.read().password)
            .body(())
            .unwrap();

        let (connection, _) = connect_async(req).await?;

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
