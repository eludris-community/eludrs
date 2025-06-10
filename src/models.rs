use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use todel::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub(crate) enum HttpResponse<T> {
    Success(T),
    Error(ErrorResponse),
}

/// An abstraction over gateway event types
#[derive(Debug, Clone)]
pub enum Event {
    /// An indication that the client has succesfully authenticated
    Authenticated,
    /// A message that has just been sent over the gateway
    Message(Message),
    /// A message that has been edit
    MessageUpdate {
        channel_id: u64,
        message_id: u64,
        data: MessageEdit,
    },
    /// A message that has been deleted
    MessageDelete { channel_id: u64, message_id: u64 },
    /// The embeds in a message have been populated
    MessageEmbedPopulate {
        channel_id: u64,
        message_id: u64,
        embeds: Vec<Embed>,
    },
    /// A users data has been updated
    UserUpdate(User),
    /// A user's status has been updated
    PresenceUpdate { user_id: u64, status: Status },
    /// This user has joined a sphere
    SphereJoin(CachedSphere),
    /// A sphere has been updated
    SphereUpdate {
        old: CachedSphere,
        data: SphereEdit,
        sphere_id: u64,
    },
    /// This user has left a sphere
    SphereLeave(CachedSphere),
    /// A user has joined a sphere
    SphereMemberJoin(CachedMember),
    /// A member updated their data in a sphere
    MemberUpdate {
        old: Member,
        data: MemberEdit,
        user_id: u64,
        sphere_id: u64,
    },
    /// A user has left a sphere
    SphereMemberLeave(CachedMember),
    /// A category has been created
    CategoryCreate { category: Category, sphere_id: u64 },
    /// A category has been edited
    CategoryUpdate {
        data: CategoryEdit,
        category_id: u64,
        sphere_id: u64,
    },
    /// A category has been deleted
    CategoryDelete { category_id: u64, sphere_id: u64 },
    /// A channel has been created
    SphereChannelCreate {
        channel: SphereChannel,
        sphere_id: u64,
    },
    /// A channel has been edited
    SphereChannelUpdate {
        data: SphereChannelEdit,
        channel_id: u64,
        sphere_id: u64,
    },
    /// A channel has been deleted
    SphereChannelDelete { channel_id: u64, sphere_id: u64 },
    /// An emoji has been created
    EmojiCreate(Emoji),
    /// An emoji has been edited
    EmojiUpdate {
        old: Emoji,
        data: EmojiEdit,
        emoji_id: u64,
        sphere_id: u64,
    },
    /// An emoji has been deleted
    EmojiDelete(Emoji),
    /// A reaction has been added to a message
    MessageReact {
        channel_id: u64,
        message_id: u64,
        user_id: u64,
        emoji: ReactionEmoji,
    },
    /// A reaction has been removed from a message
    MessageReactionDelete {
        channel_id: u64,
        message_id: u64,
        user_id: u64,
        emoji: ReactionEmoji,
    },
    /// All reactions have been removed from a message
    MessageReactionClear { channel_id: u64, message_id: u64 },
}

/// Copy of [`todel::models::Sphere`], but with a custom member and emoji models stored in maps.
/// This flattens the cache to be more efficient to update.
#[derive(Debug, Clone, PartialEq)]
pub struct CachedSphere {
    /// The spheres's ID.
    pub id: u64,
    /// The ID of the sphere's owner.
    pub owner_id: u64,
    /// The name of the sphere.
    pub name: Option<String>,
    /// The slug of the sphere.
    pub slug: String,
    /// The sphere's type.
    pub sphere_type: SphereType,
    /// The sphere's description, can be between 1 and 4096 characters.
    pub description: Option<String>,
    /// The sphere's icon. This field has to be a valid file ID in the "sphere-icons" bucket.
    pub icon: Option<u64>,
    /// The sphere's banner. This field has to be a valid file ID in the "sphere-banners" bucket.
    pub banner: Option<u64>,
    /// The sphere's badges as a bitfield.
    pub badges: u64,
    /// The categories that this sphere contains.
    pub categories: Vec<Category>,
    /// The members that are inside this sphere, as a map.
    pub members: HashMap<u64, CachedMember>,
    /// The emojis that this sphere has, as a map.
    pub emojis: HashMap<u64, Emoji>,
}

/// Copy of [`todel::models::Sphere`], but without the user field.
/// This flattens the cache to be more efficient to update.
#[derive(Debug, Clone, PartialEq)]
pub struct CachedMember {
    /// The underlying User's ID for this member.
    pub user_id: u64,
    /// The sphere to which this member belongs.
    pub sphere_id: u64,
    /// The sphere-specific nickname of this member.
    pub nickname: Option<String>,
    /// The sphere-specific avatar of this member.
    pub sphere_avatar: Option<u64>,
    /// The sphere-specific banner of this member.
    pub sphere_banner: Option<u64>,
    /// The sphere-specific bio of this member.
    pub sphere_bio: Option<String>,
    /// The sphere-specific status of this member.
    pub sphere_status: Option<String>,
}

impl From<Sphere> for CachedSphere {
    fn from(sphere: Sphere) -> Self {
        Self {
            id: sphere.id,
            owner_id: sphere.owner_id,
            name: sphere.name,
            slug: sphere.slug,
            sphere_type: sphere.sphere_type,
            description: sphere.description,
            icon: sphere.icon,
            banner: sphere.banner,
            badges: sphere.badges,
            categories: sphere.categories,
            members: sphere
                .members
                .into_iter()
                .map(|m| (m.user.id, m.into()))
                .collect(),
            emojis: sphere.emojis.into_iter().map(|e| (e.id, e)).collect(),
        }
    }
}

impl From<Member> for CachedMember {
    fn from(member: Member) -> Self {
        Self {
            user_id: member.user.id,
            sphere_id: member.sphere_id,
            nickname: member.nickname,
            sphere_avatar: member.sphere_avatar,
            sphere_banner: member.sphere_banner,
            sphere_bio: member.sphere_bio,
            sphere_status: member.sphere_status,
        }
    }
}

impl CachedMember {
    /// Converts a `CachedMember` back into a `Member` with the given user.
    pub fn into_member(self, user: User) -> Member {
        Member {
            user,
            sphere_id: self.sphere_id,
            nickname: self.nickname,
            sphere_avatar: self.sphere_avatar,
            sphere_banner: self.sphere_banner,
            sphere_bio: self.sphere_bio,
            sphere_status: self.sphere_status,
        }
    }
}
