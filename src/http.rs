use crate::{
    models::{Error, MessageResponse},
    GatewayClient, REST_URL,
};
use reqwest::Client;
use std::{fmt::Display, time::Duration};
use todel::models::{ErrorData, InstanceInfo, Message};
use tokio::time;

/// Simple Http client
#[derive(Debug)]
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

    /// Try to get the client's internal InstanceInfo or fetch it if it does not already exist
    pub async fn get_instance_info(&mut self) -> Error<&InstanceInfo> {
        if self.instance_info.is_some() {
            Ok(self.instance_info.as_ref().unwrap())
        } else {
            self.instance_info = Some(self.fetch_instance_info().await?);
            Ok(self.instance_info.as_ref().unwrap())
        }
    }

    /// Send a message supplying both an author name and content
    pub async fn send_message<T: Display, C: Display>(
        &mut self,
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
                Ok(MessageResponse::Error(err)) => match err.data {
                    Some(ErrorData::RatelimitedError(data)) => {
                        log::info!(
                            "Client got ratelimited at /messages, retrying in {}ms",
                            data.retry_after
                        );
                        time::sleep(Duration::from_millis(data.retry_after)).await;
                    }
                    Some(ErrorData::ValidationError(data)) => {
                        log::warn!(
                            "Ran into a validation error with field {}: {}",
                            data.field_name,
                            data.error
                        );
                    }
                    _ => {}
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
    pub async fn send<T: Display>(&mut self, content: T) -> Error<Message> {
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
    pub async fn create_gateway(&mut self) -> Error<Option<GatewayClient>> {
        let info = self.get_instance_info().await?;
        Ok(Some(
            GatewayClient::new().gateway_url(info.pandemonium_url.clone()),
        ))
    }
}
