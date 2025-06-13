use crate::{
    builders::{
        CreateChannel, CreateMessage, CreateSphere, EditCategory, EditChannel, EditEmoji,
        EditMember, EditMessage, EditUser, EditUserProfile, GetMessages,
    },
    models::{HttpResponse, ProxyResponse, SphereIdentifier, UserIdentifier},
    GatewayClient, REST_URL,
};
use anyhow::Result;
use reqwest::{Client, Method};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{str::FromStr, time::Duration};
use todel::models::*;
use tokio::time;

/// Simple Http client
#[derive(Debug, Clone)]
pub struct HttpClient {
    client: Client,
    instance_info: Option<InstanceInfo>,
    token: String,
    pub rest_url: String,
}

impl HttpClient {
    /// Create a new HttpClient
    pub fn new(token: &str) -> Self {
        HttpClient {
            client: Client::new(),
            instance_info: None,
            token: token.to_string(),
            rest_url: REST_URL.to_string(),
        }
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

    pub(crate) async fn request<
        T: for<'a> Deserialize<'a>,
        Q: Serialize + ?Sized,
        B: Serialize + Sized,
    >(
        &self,
        method: &str,
        path: &str,
        query: Option<&Q>,
        body: Option<B>,
    ) -> Result<HttpResponse<T>> {
        let mut builder = self
            .client
            .request(
                Method::from_str(method)?,
                format!("{}/{}", self.rest_url, path),
            )
            .header("Authorization", &self.token);
        if let Some(body) = body {
            builder = builder.json(&body);
        }
        if let Some(query) = query {
            builder = builder.query(query);
        }
        let (client, request_res) = builder.build_split();
        let request = request_res?;
        loop {
            let request = request.try_clone().expect("Could not clone request");
            match client
                .execute(request)
                .await?
                .json::<HttpResponse<T>>()
                .await
            {
                Ok(HttpResponse::Success(data)) => {
                    break Ok(HttpResponse::Success(data));
                }
                Ok(HttpResponse::Error(err)) => match err {
                    ErrorResponse::RateLimited { retry_after, .. } => {
                        log::info!(
                            "Client got ratelimited at /{}, retrying in {}ms",
                            path,
                            retry_after
                        );
                        time::sleep(Duration::from_millis(retry_after)).await;
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

    // # Channels

    /// Get a channel by its ID.
    pub async fn get_channel(&self, channel_id: u64) -> Result<SphereChannel> {
        match self
            .request::<SphereChannel, (), ()>(
                "GET",
                &format!("channels/{}", channel_id),
                None,
                None,
            )
            .await?
        {
            HttpResponse::Success(channel) => Ok(channel),
            HttpResponse::Error(err) => Err(anyhow::anyhow!("Could not get channel: {:?}", err)),
        }
    }

    // # Emojis

    /// Add a reaction to a message.
    pub async fn add_reaction(
        &self,
        channel_id: u64,
        message_id: u64,
        emoji: ReactionEmoji,
    ) -> Result<Message> {
        let emoji_ref = emoji.get_ref();
        match self
            .request::<Message, (), ReactionEmojiReference>(
                "POST",
                &format!("channels/{}/messages/{}/reactions", channel_id, message_id),
                None,
                Some(emoji_ref),
            )
            .await?
        {
            HttpResponse::Success(message) => Ok(message),
            HttpResponse::Error(err) => Err(anyhow::anyhow!("Could not add reaction: {:?}", err)),
        }
    }

    /// Remove all reactions from a message.
    pub async fn clear_reactions(&self, channel_id: u64, message_id: u64) -> Result<Message> {
        match self
            .request::<Message, (), ()>(
                "DELETE",
                &format!(
                    "channels/{}/messages/{}/reactions/clear",
                    channel_id, message_id
                ),
                None,
                None,
            )
            .await?
        {
            HttpResponse::Success(message) => Ok(message),
            HttpResponse::Error(err) => {
                Err(anyhow::anyhow!("Could not clear reactions: {:?}", err))
            }
        }
    }

    /// Remove a reaction from a message.
    pub async fn remove_reaction(
        &self,
        channel_id: u64,
        message_id: u64,
        emoji: ReactionEmoji,
    ) -> Result<Message> {
        let emoji_ref = emoji.get_ref();
        match self
            .request::<Message, (), ReactionEmojiReference>(
                "DELETE",
                &format!("channels/{}/messages/{}/reactions", channel_id, message_id),
                None,
                Some(emoji_ref),
            )
            .await?
        {
            HttpResponse::Success(message) => Ok(message),
            HttpResponse::Error(err) => {
                Err(anyhow::anyhow!("Could not remove reaction: {:?}", err))
            }
        }
    }

    /// Create an emoji within a sphere.
    pub async fn create_emoji(
        &self,
        sphere_identifier: SphereIdentifier,
        file_id: u64,
        name: String,
    ) -> Result<Emoji> {
        match self
            .request::<Emoji, (), EmojiCreate>(
                "POST",
                &format!("spheres/{}/emojis", sphere_identifier),
                None,
                Some(EmojiCreate { file_id, name }),
            )
            .await?
        {
            HttpResponse::Success(emoji) => Ok(emoji),
            HttpResponse::Error(err) => Err(anyhow::anyhow!("Could not create emoji: {:?}", err)),
        }
    }

    /// Get an emoji by its ID.
    pub async fn get_emoji(&self, emoji_id: u64) -> Result<Emoji> {
        match self
            .request::<Emoji, (), ()>("GET", &format!("emojis/{}", emoji_id), None, None)
            .await?
        {
            HttpResponse::Success(emoji) => Ok(emoji),
            HttpResponse::Error(err) => Err(anyhow::anyhow!("Could not get emoji: {:?}", err)),
        }
    }

    /// Edit an emoji.
    pub fn edit_emoji(&self, emoji_id: u64) -> EditEmoji<'_> {
        EditEmoji::new(self, emoji_id)
    }

    /// Delete an emoji.
    pub async fn delete_emoji(&self, emoji_id: u64) -> Result<()> {
        match self
            .request::<(), (), ()>("DELETE", &format!("emojis/{}", emoji_id), None, None)
            .await?
        {
            HttpResponse::Success(_) => Ok(()),
            HttpResponse::Error(err) => Err(anyhow::anyhow!("Could not delete emoji: {:?}", err)),
        }
    }

    // # Gateway

    /// Create a [`GatewayClient`] using the connected instance's instance info
    /// pandemonium url if any.
    pub async fn create_gateway(&mut self) -> Result<GatewayClient> {
        let info = self.get_instance_info().await?;
        let gateway_url = info.pandemonium_url.clone();
        Ok(GatewayClient::new(&self.token).gateway_url(gateway_url))
    }

    // # Instance

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

    // # Members

    /// Get a member.
    pub async fn get_member(
        &self,
        sphere_id: u64,
        member_identifier: UserIdentifier,
    ) -> Result<Member> {
        match self
            .request::<Member, (), ()>(
                "GET",
                &format!("spheres/{}/members/{}", sphere_id, member_identifier),
                None,
                None,
            )
            .await?
        {
            HttpResponse::Success(member) => Ok(member),
            HttpResponse::Error(err) => Err(anyhow::anyhow!("Could not get member: {:?}", err)),
        }
    }

    /// Edit a member.
    pub fn edit_member(&self, sphere_id: u64, member_identifier: UserIdentifier) -> EditMember<'_> {
        EditMember::new(self, sphere_id, member_identifier)
    }

    // # Messaging

    /// Send a message
    pub fn send_message(&self, channel_id: u64) -> CreateMessage {
        CreateMessage::new(self, channel_id)
    }

    /// Edit a message.
    pub fn edit_message(&self, channel_id: u64, message_id: u64) -> EditMessage<'_> {
        EditMessage::new(self, channel_id, message_id)
    }

    /// Delete a message.
    pub async fn delete_message(&self, channel_id: u64, message_id: u64) -> Result<()> {
        match self
            .request::<(), (), ()>(
                "DELETE",
                &format!("channels/{}/messages/{}", channel_id, message_id),
                None,
                None,
            )
            .await?
        {
            HttpResponse::Success(_) => Ok(()),
            HttpResponse::Error(err) => Err(anyhow::anyhow!("Could not delete message: {:?}", err)),
        }
    }

    /// Get a message by its ID.
    pub async fn get_message(&self, channel_id: u64, message_id: u64) -> Result<Message> {
        match self
            .request::<Message, (), ()>(
                "GET",
                &format!("channels/{}/messages/{}", channel_id, message_id),
                None,
                None,
            )
            .await?
        {
            HttpResponse::Success(message) => Ok(message),
            HttpResponse::Error(err) => Err(anyhow::anyhow!("Could not get message: {:?}", err)),
        }
    }

    /// Get messages from a channel.
    pub fn get_messages(&self, channel_id: u64) -> GetMessages<'_> {
        GetMessages::new(self, channel_id)
    }

    // # Proxy

    pub async fn proxy(&self, url: String) -> Result<ProxyResponse> {
        match self
            .request::<ProxyResponse, [(&str, String)], ()>(
                "GET",
                "format",
                Some(&[("url", url)]),
                None,
            )
            .await?
        {
            HttpResponse::Success(response) => Ok(response),
            HttpResponse::Error(err) => Err(anyhow::anyhow!("Could not proxy request: {:?}", err)),
        }
    }

    // # Sessions

    /// Create a session.
    pub async fn create_session(
        &self,
        identifier: String,
        password: String,
        platform: String,
        client: String,
    ) -> Result<SessionCreated> {
        match self
            .request::<SessionCreated, (), SessionCreate>(
                "POST",
                "sessions",
                None,
                Some(SessionCreate {
                    identifier,
                    password,
                    platform,
                    client,
                }),
            )
            .await?
        {
            HttpResponse::Success(session) => Ok(session),
            HttpResponse::Error(err) => Err(anyhow::anyhow!("Could not create session: {:?}", err)),
        }
    }

    /// Delete a session.
    pub async fn delete_session(&self, session_id: u64, password: String) -> Result<()> {
        match self
            .request::<(), (), PasswordDeleteCredentials>(
                "DELETE",
                &format!("sessions/{}", session_id),
                None,
                Some(PasswordDeleteCredentials { password }),
            )
            .await?
        {
            HttpResponse::Success(_) => Ok(()),
            HttpResponse::Error(err) => Err(anyhow::anyhow!("Could not delete session: {:?}", err)),
        }
    }

    /// Get all sessions.
    pub async fn get_sessions(&self) -> Result<Vec<Session>> {
        match self
            .request::<Vec<Session>, (), ()>("GET", "sessions", None, None)
            .await?
        {
            HttpResponse::Success(sessions) => Ok(sessions),
            HttpResponse::Error(err) => Err(anyhow::anyhow!("Could not get sessions: {:?}", err)),
        }
    }

    // # Spheres

    /// Create a category.
    pub async fn create_category(&self, sphere_id: u64, name: String) -> Result<Category> {
        match self
            .request::<Category, (), CategoryCreate>(
                "POST",
                &format!("spheres/{}/categories", sphere_id),
                None,
                Some(CategoryCreate { name }),
            )
            .await?
        {
            HttpResponse::Success(category) => Ok(category),
            HttpResponse::Error(err) => {
                Err(anyhow::anyhow!("Could not create category: {:?}", err))
            }
        }
    }

    /// Edit a category.
    pub fn edit_category(&self, sphere_id: u64, category_id: u64) -> EditCategory<'_> {
        EditCategory::new(self, sphere_id, category_id)
    }

    /// Delete a category.
    pub async fn delete_category(&self, sphere_id: u64, category_id: u64) -> Result<()> {
        match self
            .request::<(), (), ()>(
                "DELETE",
                &format!("spheres/{}/categories/{}", sphere_id, category_id),
                None,
                None,
            )
            .await?
        {
            HttpResponse::Success(_) => Ok(()),
            HttpResponse::Error(err) => {
                Err(anyhow::anyhow!("Could not delete category: {:?}", err))
            }
        }
    }

    /// Create a channel.
    pub fn create_channel(
        &self,
        sphere_id: u64,
        name: String,
        channel_type: SphereChannelType,
    ) -> CreateChannel<'_> {
        CreateChannel::new(self, sphere_id, name, channel_type)
    }

    /// Edit a channel.
    pub fn edit_channel(&self, sphere_id: u64, channel_id: u64) -> EditChannel<'_> {
        EditChannel::new(self, sphere_id, channel_id)
    }

    /// Delete a channel.
    pub async fn delete_channel(&self, sphere_id: u64, channel_id: u64) -> Result<()> {
        match self
            .request::<(), (), ()>(
                "DELETE",
                &format!("spheres/{}/channels/{}", sphere_id, channel_id),
                None,
                None,
            )
            .await?
        {
            HttpResponse::Success(_) => Ok(()),
            HttpResponse::Error(err) => Err(anyhow::anyhow!("Could not delete channel: {:?}", err)),
        }
    }

    /// Create a sphere.
    pub fn create_sphere(&self, slug: String, sphere_type: SphereType) -> CreateSphere<'_> {
        CreateSphere::new(self, slug, sphere_type)
    }

    /// Get a sphere by its ID or slug.
    pub async fn get_sphere(&self, sphere_identifier: SphereIdentifier) -> Result<Sphere> {
        match self
            .request::<Sphere, (), ()>("GET", &format!("spheres/{}", sphere_identifier), None, None)
            .await?
        {
            HttpResponse::Success(sphere) => Ok(sphere),
            HttpResponse::Error(err) => Err(anyhow::anyhow!("Could not get sphere: {:?}", err)),
        }
    }

    /// Join a sphere by its ID or slug.
    pub async fn join_sphere(&self, sphere_identifier: SphereIdentifier) -> Result<()> {
        match self
            .request::<(), (), ()>(
                "GET",
                &format!("spheres/{}/join", sphere_identifier),
                None,
                None,
            )
            .await?
        {
            HttpResponse::Success(_) => Ok(()),
            HttpResponse::Error(err) => Err(anyhow::anyhow!("Could not join sphere: {:?}", err)),
        }
    }

    // # Users

    /// Create a user.
    pub async fn create_user(
        &self,
        username: String,
        email: String,
        password: String,
    ) -> Result<User> {
        match self
            .request::<User, (), UserCreate>(
                "POST",
                "users",
                None,
                Some(UserCreate {
                    username,
                    email,
                    password,
                }),
            )
            .await?
        {
            HttpResponse::Success(user) => Ok(user),
            HttpResponse::Error(err) => Err(anyhow::anyhow!("Could not create user: {:?}", err)),
        }
    }

    /// Delete the current user.
    pub async fn delete_user(&self, password: String) -> Result<()> {
        match self
            .request::<(), (), PasswordDeleteCredentials>(
                "DELETE",
                "users",
                None,
                Some(PasswordDeleteCredentials { password }),
            )
            .await?
        {
            HttpResponse::Success(_) => Ok(()),
            HttpResponse::Error(err) => Err(anyhow::anyhow!("Could not delete user: {:?}", err)),
        }
    }

    /// Edit the current user.
    pub fn edit_user(&self, password: String) -> EditUser<'_> {
        EditUser::new(self, password)
    }

    /// Edit the current user's profile.
    pub fn edit_user_profile(&self) -> EditUserProfile<'_> {
        EditUserProfile::new(self)
    }

    /// Get a user by their ID, username, or @me for the current user.
    pub async fn get_user(&self, user_identifier: UserIdentifier) -> Result<User> {
        match self
            .request::<User, (), ()>("GET", &format!("users/{}", user_identifier), None, None)
            .await?
        {
            HttpResponse::Success(user) => Ok(user),
            HttpResponse::Error(err) => Err(anyhow::anyhow!("Could not get user: {:?}", err)),
        }
    }

    /// Verify your email address.
    pub async fn verify_user(&self, code: String) -> Result<()> {
        match self
            .request::<(), [(&str, String)], ()>(
                "POST",
                "users/verify",
                Some(&[("code", code)]),
                None,
            )
            .await?
        {
            HttpResponse::Success(_) => Ok(()),
            HttpResponse::Error(err) => Err(anyhow::anyhow!("Could not verify user: {:?}", err)),
        }
    }

    /// Resend the verification email.
    pub async fn resend_verification(&self) -> Result<()> {
        match self
            .request::<(), (), u8>("POST", "users/resend-verification", None, None)
            .await?
        {
            HttpResponse::Success(_) => Ok(()),
            HttpResponse::Error(err) => Err(anyhow::anyhow!(
                "Could not resend verification email: {:?}",
                err
            )),
        }
    }

    /// Create password reset code.
    pub async fn create_password_reset(&self, email: String) -> Result<()> {
        match self
            .request::<(), (), serde_json::Value>(
                "POST",
                "users/reset-password",
                None,
                Some(json!({ "email": email })),
            )
            .await?
        {
            HttpResponse::Success(_) => Ok(()),
            HttpResponse::Error(err) => Err(anyhow::anyhow!(
                "Could not create password reset code: {:?}",
                err
            )),
        }
    }

    /// Reset your password.
    pub async fn reset_password(&self, code: u32, email: String, password: String) -> Result<()> {
        match self
            .request::<(), (), PasswordReset>(
                "POST",
                "users/reset-password",
                None,
                Some(PasswordReset {
                    code,
                    email,
                    password,
                }),
            )
            .await?
        {
            HttpResponse::Success(_) => Ok(()),
            HttpResponse::Error(err) => Err(anyhow::anyhow!("Could not reset password: {:?}", err)),
        }
    }
}
