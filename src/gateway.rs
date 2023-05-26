use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll, Waker},
    thread,
    time::Duration,
};

use anyhow::{bail, Result};
use futures::{stream::SplitStream, SinkExt, Stream, StreamExt};
use rand::{rngs::StdRng, Rng, SeedableRng};
use todel::models::{ClientPayload, Message, ServerPayload};
use tokio::{net::TcpStream, sync::Mutex, task::JoinHandle, time};
use tokio_tungstenite::{
    connect_async, tungstenite::Message as WSMessage, MaybeTlsStream, WebSocketStream,
};

use crate::GATEWAY_URL;

type WsReceiver = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

/// A Stream of Pandemonium events
#[derive(Debug, Clone)]
pub struct Events {
    gateway_url: String,
    rx: Arc<Mutex<Option<WsReceiver>>>,
    ping: Arc<Mutex<Option<JoinHandle<()>>>>,
    rng: Arc<Mutex<StdRng>>,
}

/// Simple gateway client
#[derive(Debug, Clone)]
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
    pub async fn get_events(&self) -> Result<Events> {
        let mut events = Events::new(self.gateway_url.to_string());
        events.connect().await?;
        Ok(events)
    }
}

impl Events {
    fn new(gateway_url: String) -> Self {
        Self {
            gateway_url,
            rx: Arc::new(Mutex::new(None)),
            ping: Arc::new(Mutex::new(None)),
            rng: Arc::new(Mutex::new(StdRng::from_entropy())),
        }
    }

    async fn connect(&mut self) -> Result<()> {
        log::debug!("Events connecting");
        let mut ping = self.ping.lock().await;
        if ping.is_some() {
            ping.as_mut().unwrap().abort();
        }
        let (socket, _) = connect_async(&self.gateway_url).await?;
        let (mut tx, mut rx) = socket.split();
        loop {
            if let Some(Ok(WSMessage::Text(msg))) = rx.next().await {
                if let Ok(ServerPayload::Hello {
                    heartbeat_interval, ..
                }) = serde_json::from_str(&msg)
                {
                    time::sleep(Duration::from_millis(
                        self.rng.lock().await.gen_range(0..heartbeat_interval),
                    ))
                    .await;
                    *ping = Some(tokio::spawn(async move {
                        loop {
                            match tx
                                .send(WSMessage::Text(
                                    serde_json::to_string(&ClientPayload::Ping).unwrap(),
                                ))
                                .await
                            {
                                Ok(_) => {
                                    time::sleep(Duration::from_millis(heartbeat_interval)).await
                                }
                                Err(err) => {
                                    log::debug!("Encountered error while pinging {:?}", err);
                                    break;
                                }
                            }
                        }
                    }));
                    break;
                }
            } else {
                bail!("Could not find HELLO payload");
            }
        }

        *self.rx.lock().await = Some(rx);
        Ok(())
    }

    async fn reconect(
        waker: Waker,
        gateway_url: String,
        rx: Arc<Mutex<Option<WsReceiver>>>,
        ping: Arc<Mutex<Option<JoinHandle<()>>>>,
        rng: Arc<Mutex<StdRng>>,
    ) {
        let mut wait = 1;
        let mut ping = ping.lock().await;
        if ping.is_some() {
            ping.as_mut().unwrap().abort();
        }
        'outer: loop {
            match connect_async(&gateway_url).await {
                Ok((socket, _)) => {
                    let (mut tx, mut new_rx) = socket.split();
                    loop {
                        if let Some(Ok(WSMessage::Text(msg))) = new_rx.next().await {
                            if let Ok(ServerPayload::Hello {
                                heartbeat_interval, ..
                            }) = serde_json::from_str(&msg)
                            {
                                time::sleep(Duration::from_millis(
                                    rng.lock().await.gen_range(0..heartbeat_interval),
                                ))
                                .await;
                                *ping = Some(tokio::spawn(async move {
                                    loop {
                                        match tx
                                            .send(WSMessage::Text(
                                                serde_json::to_string(&ClientPayload::Ping)
                                                    .unwrap(),
                                            ))
                                            .await
                                        {
                                            Ok(_) => {
                                                time::sleep(Duration::from_millis(
                                                    heartbeat_interval,
                                                ))
                                                .await
                                            }
                                            Err(err) => {
                                                log::debug!(
                                                    "Encountered error while pinging {:?}",
                                                    err
                                                );
                                                break;
                                            }
                                        }
                                    }
                                }));
                                break;
                            }
                        } else {
                            log::error!("Could not find HELLO payload");
                            continue 'outer;
                        }
                    }

                    *rx.lock().await = Some(new_rx);
                    log::debug!("Reconnected to websocket");
                    break;
                }
                Err(err) => {
                    log::info!(
                        "Websocket reconnection failed {}, trying again in {}s",
                        err,
                        wait
                    );
                    thread::sleep(Duration::from_secs(wait));
                    if wait < 64 {
                        wait *= 2;
                    }
                }
            }
        }
        waker.wake();
    }
}

impl Stream for Events {
    type Item = Message;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            let mut rx = futures::executor::block_on(async { self.rx.lock().await });
            if rx.is_some() {
                match rx.as_mut().unwrap().poll_next_unpin(cx) {
                    Poll::Ready(Some(Ok(msg))) => match msg {
                        WSMessage::Text(msg) => {
                            if let Ok(ServerPayload::MessageCreate(msg)) =
                                serde_json::from_str(&msg)
                            {
                                break Poll::Ready(Some(msg));
                            }
                        }
                        WSMessage::Close(_) => {
                            log::debug!("Websocket closed, reconnecting");
                            tokio::spawn(Events::reconect(
                                cx.waker().clone(),
                                self.gateway_url.clone(),
                                Arc::clone(&self.rx),
                                Arc::clone(&self.ping),
                                Arc::clone(&self.rng),
                            ));
                            return Poll::Pending;
                        }
                        _ => {}
                    },
                    Poll::Pending => break Poll::Pending,
                    Poll::Ready(None) => {
                        log::debug!("Websocket closed, reconnecting");
                        tokio::spawn(Events::reconect(
                            cx.waker().clone(),
                            self.gateway_url.clone(),
                            Arc::clone(&self.rx),
                            Arc::clone(&self.ping),
                            Arc::clone(&self.rng),
                        ));
                        return Poll::Pending;
                    }
                    _ => {}
                }
            }
        }
    }
}
