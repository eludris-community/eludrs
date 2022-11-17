use crate::models::Error;
use reqwest::Client;
use std::fmt::Display;
use todel::models::{Info, Message};

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
    pub async fn fetch_instance_info(&self) -> Error<Info> {
        Ok(self.client.get(&self.rest_url).send().await?.json().await?)
    }

    /// Send a message supplying both an author name and content
    pub async fn send_message<T: Display, C: Display>(
        &self,
        author: T,
        content: C,
    ) -> Error<Message> {
        Ok(self
            .client
            .post(format!("{}/messages", self.rest_url))
            .json(&Message {
                author: author.to_string(),
                content: content.to_string(),
            })
            .send()
            .await?
            .json()
            .await?)
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
}
