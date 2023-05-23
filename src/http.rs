use crate::{models::MessageResponse, GatewayClient, REST_URL};
use anyhow::Result;
use reqwest::Client;
use std::{fmt::Display, time::Duration};
use todel::models::{ErrorResponse, InstanceInfo, Message};
use tokio::time;

/// Simple Http client
#[derive(Debug, Clone)]
pub struct HttpClient {
    client: Client,
    instance_info: Option<InstanceInfo>,
    pub rest_url: String,
    pub user_name: Option<String>,
}

impl Default for HttpClient {
    fn default() -> Self {
        HttpClient {
            client: Client::new(),
            instance_info: None,
            rest_url: REST_URL.to_string(),
            user_name: None,
        }
    }
}

impl HttpClient {
    /// Create a new HttpClient
    pub fn new() -> Self {
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
    pub fn name(mut self, name: String) -> Self {
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
    pub fn rest_url(mut self, url: String) -> Self {
        self.rest_url = url;
        self
    }

    /// Fetch the info payload of an instance
    pub async fn fetch_instance_info(&self) -> Result<InstanceInfo> {
        Ok(self.client.get(&self.rest_url).send().await?.json().await?)
    }

    /// Try to get the client's internal InstanceInfo or fetch it if it does not already exist
    pub async fn get_instance_info(&mut self) -> Result<&InstanceInfo> {
        if self.instance_info.is_some() {
            Ok(self.instance_info.as_ref().unwrap())
        } else {
            self.instance_info = Some(self.fetch_instance_info().await?);
            Ok(self.instance_info.as_ref().unwrap())
        }
    }

    /// Send a message supplying both an author name and content
    pub async fn send_message<T: Display, C: Display>(
        &self,
        author: T,
        content: C,
    ) -> Result<Message> {
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
                Ok(MessageResponse::Error(err)) => match err {
                    ErrorResponse::RateLimited { try_after, .. } => {
                        log::info!(
                            "Client got ratelimited at /messages, retrying in {}ms",
                            try_after
                        );
                        time::sleep(Duration::from_millis(try_after)).await;
                    }
                    ErrorResponse::Validation {
                        value_name, info, ..
                    } => {
                        Err(anyhow::anyhow!(
                            "Ran into a validation error with field {}: {}",
                            value_name,
                            info,
                        ))?;
                    }
                    err => Err(anyhow::anyhow!("Could not send message: {:?}", err))?,
                },
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
    pub async fn send<T: Display>(&self, content: T) -> Result<Message> {
        self.send_message(
            &self
                .user_name
                .as_ref()
                .expect("You have to specifiy a name to run this function"),
            content,
        )
        .await
    }

    /// Create a [`GatewayClient`] using the connected instance's instance info
    /// pandemonium url if any.
    pub async fn create_gateway(&mut self) -> Result<GatewayClient> {
        let info = self.get_instance_info().await?;
        Ok(GatewayClient::new().gateway_url(info.pandemonium_url.clone()))
    }
}
