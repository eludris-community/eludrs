use std::{
    future::{Future, IntoFuture},
    pin::Pin,
};

use anyhow::Result;
use todel::models::{
    Category, CategoryEdit, CustomEmbed, Emoji, EmojiEdit, Member, MemberEdit, Message,
    MessageCreate, MessageDisguise, MessageEdit, Sphere, SphereChannel, SphereChannelCreate,
    SphereChannelEdit, SphereChannelType, SphereCreate, SphereType, StatusType, User, UserEdit,
    UserProfileEdit,
};

use crate::{
    models::{HttpResponse, UserIdentifier},
    HttpClient,
};

#[must_use]
pub struct EditEmoji<'a> {
    http: &'a HttpClient,
    emoji_id: u64,
    data: EmojiEdit,
}

impl<'a> EditEmoji<'a> {
    pub(crate) fn new(http: &'a HttpClient, emoji_id: u64) -> Self {
        Self {
            http,
            emoji_id,
            data: EmojiEdit {
                name: String::new(),
            },
        }
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.data.name = name.into();
        self
    }
}

impl<'a> IntoFuture for EditEmoji<'a> {
    type Output = Result<Emoji>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + 'a>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            match self
                .http
                .request::<Emoji, (), EmojiEdit>(
                    "PATCH",
                    &format!("emojis/{}", self.emoji_id),
                    None,
                    Some(self.data),
                )
                .await?
            {
                HttpResponse::Success(emoji) => Ok(emoji),
                HttpResponse::Error(err) => Err(anyhow::anyhow!("Could not edit emoji: {:?}", err)),
            }
        })
    }
}

#[must_use]
pub struct EditMember<'a> {
    http: &'a HttpClient,
    guild_id: u64,
    member_identifier: UserIdentifier,
    data: MemberEdit,
}

impl<'a> EditMember<'a> {
    pub(crate) fn new(
        http: &'a HttpClient,
        guild_id: u64,
        member_identifier: UserIdentifier,
    ) -> Self {
        Self {
            http,
            guild_id,
            member_identifier,
            data: MemberEdit {
                nickname: None,
                sphere_avatar: None,
                sphere_banner: None,
                sphere_bio: None,
                sphere_status: None,
            },
        }
    }

    pub fn nickname(mut self, nickname: Option<impl Into<String>>) -> Self {
        self.data.nickname = Some(nickname.map(Into::into));
        self
    }

    pub fn sphere_avatar(mut self, avatar: Option<u64>) -> Self {
        self.data.sphere_avatar = Some(avatar);
        self
    }

    pub fn sphere_banner(mut self, banner: Option<u64>) -> Self {
        self.data.sphere_banner = Some(banner);
        self
    }

    pub fn sphere_bio(mut self, bio: Option<impl Into<String>>) -> Self {
        self.data.sphere_bio = Some(bio.map(Into::into));
        self
    }

    pub fn sphere_status(mut self, status: Option<impl Into<String>>) -> Self {
        self.data.sphere_status = Some(status.map(Into::into));
        self
    }
}

impl<'a> IntoFuture for EditMember<'a> {
    type Output = Result<Member>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + 'a>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            match self
                .http
                .request::<Member, (), MemberEdit>(
                    "PATCH",
                    &format!(
                        "guilds/{}/members/{}",
                        self.guild_id, self.member_identifier
                    ),
                    None,
                    Some(self.data),
                )
                .await?
            {
                HttpResponse::Success(member) => Ok(member),
                HttpResponse::Error(err) => {
                    Err(anyhow::anyhow!("Could not edit member: {:?}", err))
                }
            }
        })
    }
}

#[must_use]
pub struct CreateMessage<'a> {
    http: &'a HttpClient,
    channel_id: u64,
    data: MessageCreate,
}

impl<'a> CreateMessage<'a> {
    pub(crate) fn new(http: &'a HttpClient, channel_id: u64) -> Self {
        Self {
            http,
            channel_id,
            data: MessageCreate {
                content: None,
                attachments: vec![],
                embeds: vec![],
                disguise: None,
                reference: None,
            },
        }
    }

    pub fn attachments(mut self, attachments: Vec<u64>) -> Self {
        self.data.attachments = attachments;
        self
    }

    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.data.content = Some(content.into());
        self
    }

    pub fn disguise(mut self, disguise: MessageDisguise) -> Self {
        self.data.disguise = Some(disguise);
        self
    }

    pub fn embeds(mut self, embeds: Vec<CustomEmbed>) -> Self {
        self.data.embeds = embeds;
        self
    }

    pub fn reference(mut self, reference: u64) -> Self {
        self.data.reference = Some(reference);
        self
    }
}

impl<'a> IntoFuture for CreateMessage<'a> {
    type Output = Result<Message>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + 'a>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            match self
                .http
                .request::<Message, (), MessageCreate>(
                    "POST",
                    &format!("channels/{}/messages", self.channel_id),
                    None,
                    Some(self.data),
                )
                .await?
            {
                HttpResponse::Success(data) => Ok(data),
                HttpResponse::Error(err) => {
                    Err(anyhow::anyhow!("Could not send message: {:?}", err))
                }
            }
        })
    }
}

#[must_use]
pub struct EditMessage<'a> {
    http: &'a HttpClient,
    channel_id: u64,
    message_id: u64,
    data: MessageEdit,
}

impl<'a> EditMessage<'a> {
    pub(crate) fn new(http: &'a HttpClient, channel_id: u64, message_id: u64) -> Self {
        Self {
            http,
            channel_id,
            message_id,
            data: MessageEdit {
                content: None,
                attachments: None,
                embeds: None,
            },
        }
    }

    pub fn attachments(mut self, attachments: Vec<u64>) -> Self {
        self.data.attachments = Some(attachments);
        self
    }

    pub fn content(mut self, content: Option<impl Into<String>>) -> Self {
        self.data.content = Some(content.map(Into::into));
        self
    }

    pub fn embeds(mut self, embeds: Vec<CustomEmbed>) -> Self {
        self.data.embeds = Some(embeds);
        self
    }
}

impl<'a> IntoFuture for EditMessage<'a> {
    type Output = Result<Message>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + 'a>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            match self
                .http
                .request::<Message, (), MessageEdit>(
                    "PATCH",
                    &format!("channels/{}/messages/{}", self.channel_id, self.message_id),
                    None,
                    Some(self.data),
                )
                .await?
            {
                HttpResponse::Success(message) => Ok(message),
                HttpResponse::Error(err) => {
                    Err(anyhow::anyhow!("Could not edit message: {:?}", err))
                }
            }
        })
    }
}

#[must_use]
pub struct GetMessages<'a> {
    http: &'a HttpClient,
    channel_id: u64,
    limit: Option<u32>,
    before: Option<u64>,
    after: Option<u64>,
}

impl<'a> GetMessages<'a> {
    pub(crate) fn new(http: &'a HttpClient, channel_id: u64) -> Self {
        Self {
            http,
            channel_id,
            limit: None,
            before: None,
            after: None,
        }
    }

    pub fn after(mut self, after: u64) -> Self {
        self.after = Some(after);
        self
    }

    pub fn before(mut self, before: u64) -> Self {
        self.before = Some(before);
        self
    }

    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }
}

impl<'a> IntoFuture for GetMessages<'a> {
    type Output = Result<Vec<Message>>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + 'a>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let mut query = vec![];
            if let Some(before) = self.before {
                query.push(("before", before.to_string()));
            }
            if let Some(after) = self.after {
                query.push(("after", after.to_string()));
            }
            if let Some(limit) = self.limit {
                query.push(("limit", limit.to_string()));
            }
            match self
                .http
                .request::<Vec<Message>, Vec<(&str, String)>, ()>(
                    "GET",
                    &format!("channels/{}/messages", self.channel_id),
                    Some(&query),
                    None,
                )
                .await?
            {
                HttpResponse::Success(messages) => Ok(messages),
                HttpResponse::Error(err) => {
                    Err(anyhow::anyhow!("Could not get messages: {:?}", err))
                }
            }
        })
    }
}

#[must_use]
pub struct EditCategory<'a> {
    http: &'a HttpClient,
    sphere_id: u64,
    category_id: u64,
    data: CategoryEdit,
}

impl<'a> EditCategory<'a> {
    pub(crate) fn new(http: &'a HttpClient, sphere_id: u64, category_id: u64) -> Self {
        Self {
            http,
            sphere_id,
            category_id,
            data: CategoryEdit {
                name: None,
                position: None,
            },
        }
    }
}

impl<'a> EditCategory<'a> {
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.data.name = Some(name.into());
        self
    }

    pub fn position(mut self, position: u32) -> Self {
        self.data.position = Some(position);
        self
    }
}

impl<'a> IntoFuture for EditCategory<'a> {
    type Output = Result<Category>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + 'a>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            match self
                .http
                .request::<Category, (), CategoryEdit>(
                    "PATCH",
                    &format!("spheres/{}/categories/{}", self.sphere_id, self.category_id),
                    None,
                    Some(self.data),
                )
                .await?
            {
                HttpResponse::Success(category) => Ok(category),
                HttpResponse::Error(err) => {
                    Err(anyhow::anyhow!("Could not edit category: {:?}", err))
                }
            }
        })
    }
}

#[must_use]
pub struct CreateChannel<'a> {
    http: &'a HttpClient,
    sphere_id: u64,
    data: SphereChannelCreate,
}

impl<'a> CreateChannel<'a> {
    pub(crate) fn new(
        http: &'a HttpClient,
        sphere_id: u64,
        name: String,
        channel_type: SphereChannelType,
    ) -> Self {
        Self {
            http,
            sphere_id,
            data: SphereChannelCreate {
                name: name,
                channel_type: channel_type,
                topic: None,
                category_id: None,
            },
        }
    }

    pub fn category_id(mut self, category_id: u64) -> Self {
        self.data.category_id = Some(category_id);
        self
    }

    pub fn topic(mut self, topic: impl Into<String>) -> Self {
        self.data.topic = Some(topic.into());
        self
    }
}

impl<'a> IntoFuture for CreateChannel<'a> {
    type Output = Result<SphereChannel>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + 'a>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            match self
                .http
                .request::<SphereChannel, (), SphereChannelCreate>(
                    "POST",
                    &format!("spheres/{}/channels", self.sphere_id),
                    None,
                    Some(self.data),
                )
                .await?
            {
                HttpResponse::Success(channel) => Ok(channel),
                HttpResponse::Error(err) => {
                    Err(anyhow::anyhow!("Could not create channel: {:?}", err))
                }
            }
        })
    }
}

#[must_use]
pub struct EditChannel<'a> {
    http: &'a HttpClient,
    sphere_id: u64,
    channel_id: u64,
    data: SphereChannelEdit,
}

impl<'a> EditChannel<'a> {
    pub(crate) fn new(http: &'a HttpClient, sphere_id: u64, channel_id: u64) -> Self {
        Self {
            http,
            sphere_id,
            channel_id,
            data: SphereChannelEdit {
                name: None,
                topic: None,
                position: None,
                category_id: None,
            },
        }
    }

    pub fn category_id(mut self, category_id: u64) -> Self {
        self.data.category_id = Some(category_id);
        self
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.data.name = Some(name.into());
        self
    }

    pub fn position(mut self, position: u32) -> Self {
        self.data.position = Some(position);
        self
    }

    pub fn topic(mut self, topic: Option<impl Into<String>>) -> Self {
        self.data.topic = Some(topic.map(Into::into));
        self
    }
}

impl<'a> IntoFuture for EditChannel<'a> {
    type Output = Result<SphereChannel>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + 'a>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            match self
                .http
                .request::<SphereChannel, (), SphereChannelEdit>(
                    "PATCH",
                    &format!("spheres/{}/channels/{}", self.sphere_id, self.channel_id),
                    None,
                    Some(self.data),
                )
                .await?
            {
                HttpResponse::Success(channel) => Ok(channel),
                HttpResponse::Error(err) => {
                    Err(anyhow::anyhow!("Could not edit channel: {:?}", err))
                }
            }
        })
    }
}

#[must_use]
pub struct CreateSphere<'a> {
    http: &'a HttpClient,
    data: SphereCreate,
}

impl<'a> CreateSphere<'a> {
    pub(crate) fn new(http: &'a HttpClient, slug: String, sphere_type: SphereType) -> Self {
        Self {
            http,
            data: SphereCreate {
                slug,
                sphere_type,
                name: None,
                description: None,
                icon: None,
                banner: None,
            },
        }
    }

    pub fn banner(mut self, banner: u64) -> Self {
        self.data.banner = Some(banner);
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.data.description = Some(description.into());
        self
    }

    pub fn icon(mut self, icon: u64) -> Self {
        self.data.icon = Some(icon);
        self
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.data.name = Some(name.into());
        self
    }
}

impl<'a> IntoFuture for CreateSphere<'a> {
    type Output = Result<Sphere>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + 'a>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            match self
                .http
                .request::<Sphere, (), SphereCreate>("POST", "spheres", None, Some(self.data))
                .await?
            {
                HttpResponse::Success(sphere) => Ok(sphere),
                HttpResponse::Error(err) => {
                    Err(anyhow::anyhow!("Could not create sphere: {:?}", err))
                }
            }
        })
    }
}

#[must_use]
pub struct EditUser<'a> {
    http: &'a HttpClient,
    data: UserEdit,
}

impl<'a> EditUser<'a> {
    pub(crate) fn new(http: &'a HttpClient, password: String) -> Self {
        Self {
            http,
            data: UserEdit {
                password: password,
                username: None,
                email: None,
                new_password: None,
            },
        }
    }

    pub fn email(mut self, email: impl Into<String>) -> Self {
        self.data.email = Some(email.into());
        self
    }

    pub fn new_password(mut self, new_password: impl Into<String>) -> Self {
        self.data.new_password = Some(new_password.into());
        self
    }

    pub fn username(mut self, username: impl Into<String>) -> Self {
        self.data.username = Some(username.into());
        self
    }
}

impl<'a> IntoFuture for EditUser<'a> {
    type Output = Result<User>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + 'a>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            match self
                .http
                .request::<User, (), UserEdit>("PATCH", "users", None, Some(self.data))
                .await?
            {
                HttpResponse::Success(user) => Ok(user),
                HttpResponse::Error(err) => {
                    Err(anyhow::anyhow!("Could not update user: {:?}", err))
                }
            }
        })
    }
}

#[must_use]
pub struct EditUserProfile<'a> {
    http: &'a HttpClient,
    data: UserProfileEdit,
}

impl<'a> EditUserProfile<'a> {
    pub(crate) fn new(http: &'a HttpClient) -> Self {
        Self {
            http,
            data: UserProfileEdit {
                avatar: None,
                banner: None,
                bio: None,
                display_name: None,
                status: None,
                status_type: None,
            },
        }
    }

    pub fn avatar(mut self, avatar: Option<u64>) -> Self {
        self.data.avatar = Some(avatar);
        self
    }

    pub fn banner(mut self, banner: Option<u64>) -> Self {
        self.data.banner = Some(banner);
        self
    }

    pub fn bio(mut self, bio: Option<impl Into<String>>) -> Self {
        self.data.bio = Some(bio.map(Into::into));
        self
    }

    pub fn display_name(mut self, display_name: Option<impl Into<String>>) -> Self {
        self.data.display_name = Some(display_name.map(Into::into));
        self
    }

    pub fn status(mut self, status: Option<impl Into<String>>) -> Self {
        self.data.status = Some(status.map(Into::into));
        self
    }

    pub fn status_type(mut self, status_type: StatusType) -> Self {
        self.data.status_type = Some(status_type);
        self
    }
}

impl<'a> IntoFuture for EditUserProfile<'a> {
    type Output = Result<User>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + 'a>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            match self.http
                .request::<User, (), UserProfileEdit>("PATCH", "users/profile", None, Some(self.data))
                .await?
            {
                HttpResponse::Success(user) => Ok(user),
                HttpResponse::Error(err) => {
                    Err(anyhow::anyhow!("Could not update user profile: {:?}", err))
                }
            }
        })
    }
}
