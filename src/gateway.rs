use std::{
    pin::Pin,
    task::{Context, Poll},
    thread,
    time::Duration,
};

use futures::{stream::SplitStream, SinkExt, Stream, StreamExt};
use todel::models::Message;
use tokio::{net::TcpStream, task::JoinHandle, time};
use tokio_tungstenite::{
    connect_async, tungstenite::Message as WSMessage, MaybeTlsStream, WebSocketStream,
};

use crate::models::Error;

/// The default gateway url
pub const GATEWAY_URL: &str = "wss://eludris.tooty.xyz/ws/";

/// A Stream of Pandemonium events
#[derive(Debug)]
pub struct Events {
    gateway_url: String,
    rx: Option<SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>>,
    ping: Option<JoinHandle<()>>,
}

/// Simple gateway client
#[derive(Debug)]
pub struct GatewayClient {
    pub gateway_url: String,
}

impl Default for GatewayClient {
    fn default() -> Self {
        GatewayClient {
            gateway_url: GATEWAY_URL.to_string(),
        }
    }
}

impl GatewayClient {
    /// Create a new GatewayClient
    pub fn new() -> GatewayClient {
        GatewayClient::default()
    }
    /// Change the url of the GatewayClient
    ///
    /// # Example:
    /// ```rust
    /// use eludrs::GatewayClient;
    ///
    /// let client = GatewayClient::new().gateway_url("http://0.0.0.0:7160".to_string());
    ///
    /// assert_eq!(client.gateway_url, "http://0.0.0.0:7160".to_string())
    /// ```
    pub fn gateway_url(mut self, url: String) -> GatewayClient {
        self.gateway_url = url;
        self
    }

    /// Start a connection to the Pandemonium and return [`Events`]
    pub async fn get_events(&self) -> Error<Events> {
        let mut events = Events::new(self.gateway_url.to_string());
        events.connect().await?;
        Ok(events)
    }
}

impl Events {
    fn new(gateway_url: String) -> Self {
        Self {
            gateway_url,
            rx: None,
            ping: None,
        }
    }

    async fn connect(&mut self) -> Error<()> {
        log::debug!("Events connecting");
        if let Some(ping) = &self.ping {
            ping.abort();
        }
        let (socket, _) = connect_async(&self.gateway_url).await?;
        let (mut tx, rx) = socket.split();
        self.ping = Some(tokio::spawn(async move {
            loop {
                match tx.send(WSMessage::Ping(vec![])).await {
                    Ok(_) => time::sleep(Duration::from_secs(20)).await,
                    Err(err) => {
                        log::debug!("Encountered error while pinging {:?}", err);
                        break;
                    }
                }
            }
        }));
        self.rx = Some(rx);
        Ok(())
    }
}

impl Stream for Events {
    type Item = Message;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            match &mut self.rx {
                Some(rx) => match rx.poll_next_unpin(cx) {
                    Poll::Ready(Some(Ok(msg))) => match msg {
                        WSMessage::Text(msg) => {
                            if let Ok(msg) = serde_json::from_str(&msg) {
                                break Poll::Ready(Some(msg));
                            }
                        }
                        WSMessage::Close(_) => {
                            log::debug!("Websocket closed, reconnecting");
                            let mut wait = 1;
                            loop {
                                if futures::executor::block_on(async { self.connect().await })
                                    .is_err()
                                {
                                    log::info!(
                                        "Websocket reconnection failed, trying again in {}s",
                                        wait
                                    );
                                    thread::sleep(Duration::from_secs(wait));
                                    wait *= 2;
                                } else {
                                    log::debug!("Reconnected to websocket");
                                    break;
                                }
                            }
                        }
                        _ => {}
                    },
                    Poll::Pending => break Poll::Pending,
                    Poll::Ready(None) => {
                        log::debug!("Websocket closed, reconnecting");
                        let mut wait = 1;
                        loop {
                            if futures::executor::block_on(async { self.connect().await }).is_err()
                            {
                                log::info!(
                                    "Websocket reconnection failed, trying again in {}s",
                                    wait
                                );
                                thread::sleep(Duration::from_secs(wait));
                                if wait < 64 {
                                    wait *= 2;
                                }
                            } else {
                                log::debug!("Reconnected to websocket");
                                break;
                            }
                        }
                    }
                    _ => {}
                },
                None => unreachable!(),
            }
        }
    }
}
