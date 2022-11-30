use crate::{
    models::{Error, MessageResponse},
    GatewayClient,
};
use reqwest::Client;
use std::{fmt::Display, time::Duration};
use todel::models::{InstanceInfo, Message};
use tokio::time;

/// The default rest url
pub const REST_URL: &str = "https://eludris.tooty.xyz";

/// Simple Http client
#[derive(Debug)]
pub struct HttpClient {
    client: Client,
    pub rest_url: String,
    pub user_name: Option<String>,
}

impl Default for HttpClient {
    fn default() -> Self {
        HttpClient {
            client: Client::new(),
            rest_url: REST_URL.to_string(),
            user_name: None,
        }
    }
}

impl HttpClient {
    /// Create a new HttpClient
    pub fn new() -> HttpClient {
        HttpClient::default()
    }

    /// Change the [`HttpClient::user_name`] of the HttpClient
    ///
    /// # Example:
    /// ```rust
    /// use eludrs::HttpClient;
    ///
    /// let client = HttpClient::new().name("Uwuki".to_string());
    ///
    /// assert_eq!(client.user_name, Some("Uwuki".to_string()))
    /// ```
    pub fn name(mut self, name: String) -> HttpClient {
        self.user_name = Some(name);
        self
    }

    /// Change the url of the HttpClient
    ///
    /// # Example:
    /// ```rust
    /// use eludrs::HttpClient;
    ///
    /// let client = HttpClient::new().rest_url("http://0.0.0.0:7159".to_string());
    ///
    /// assert_eq!(client.rest_url, "http://0.0.0.0:7159".to_string())
    /// ```
    pub fn rest_url(mut self, url: String) -> HttpClient {
        self.rest_url = url;
        self
    }

    /// Fetch the info payload of an instance
    pub async fn fetch_instance_info(&self) -> Error<InstanceInfo> {
        Ok(self.client.get(&self.rest_url).send().await?.json().await?)
    }

    /// Send a message supplying both an author name and content
    pub async fn send_message<T: Display, C: Display>(
        &self,
        author: T,
        content: C,
    ) -> Error<Message> {
        loop {
            match self
                .client
                .post(format!("{}/messages", self.rest_url))
                .json(&Message {
                    author: author.to_string(),
                    content: content.to_string(),
                })
                .send()
                .await?
                .json::<MessageResponse>()
                .await
            {
                Ok(MessageResponse::Message(msg)) => {
                    break Ok(msg);
                }
                Ok(MessageResponse::Ratelimited(data)) => {
                    log::info!(
                        "Client got ratelimited at /messages, retrying in {}ms",
                        data.data.retry_after
                    );
                    time::sleep(Duration::from_millis(data.data.retry_after)).await;
                }
                Err(err) => {
                    break Err(err)?;
                }
            }
        }
    }

    /// Send a message using the client's [`HttpClient::user_name`]
    ///
    /// # Panics
    ///
    /// This function can panic if there is no name set by the [`HttpClient::name`] function
    pub async fn send<T: Display>(&self, content: T) -> Error<Message> {
        self.send_message(
            &self
                .user_name
                .clone()
                .expect("You have to specifiy a name to run this function"),
            content,
        )
        .await
    }

    /// Create a [`GatewayClient`] using the connected instance's instance info
    /// pandemonium url if any.
    pub async fn create_gateway(&self) -> Error<Option<GatewayClient>> {
        let info = self.fetch_instance_info().await?;
        match info.pandemonium_url {
            Some(url) => Ok(Some(GatewayClient::new().gateway_url(url))),
            None => Ok(None),
        }
    }
}
