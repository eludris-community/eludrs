use serde::{Deserialize, Serialize};
use todel::models::{
    Category, CategoryEdit, Emoji, ErrorResponse, Member, Message, Sphere, SphereChannel,
    SphereChannelEdit, SphereType, Status, User,
};

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
    /// A users data has been updated
    UserUpdate(User),
    /// A user's status has been updated
    PresenceUpdate { user_id: u64, status: Status },
    /// This user has joined a sphere
    SphereJoin(CachedSphere),
    /// A user has joined a sphere
    SphereMemberJoin { user: User, sphere_id: u64 },
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
}

/// Copy of [`todel::models::Sphere`], but with a custom member model.
/// This flattens the cache to be more efficient to update.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CachedSphere {
    /// The spheres's ID.
    pub id: u64,
    /// The ID of the sphere's owner.
    pub owner_id: u64,
    /// The name of the sphere.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The slug of the sphere.
    pub slug: String,
    /// The sphere's type.
    #[serde(rename = "type")]
    pub sphere_type: SphereType,
    /// The sphere's description, can be between 1 and 4096 characters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The sphere's icon. This field has to be a valid file ID in the "sphere-icons" bucket.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<u64>,
    /// The sphere's banner. This field has to be a valid file ID in the "sphere-banners" bucket.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub banner: Option<u64>,
    /// The sphere's badges as a bitfield.
    pub badges: u64,
    /// The categories that this sphere contains.
    pub categories: Vec<Category>,
    /// The members that are inside this sphere.
    pub members: Vec<CachedMember>,
    /// The emojis that this sphere has.
    pub emojis: Vec<Emoji>,
}

/// Copy of [`todel::models::Sphere`], but without the user field.
/// This flattens the cache to be more efficient to update.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CachedMember {
    /// The underlying User's ID for this member.
    pub user_id: u64,
    /// The sphere to which this member belongs.
    pub sphere_id: u64,
    /// The sphere-specific nickname of this member.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nickname: Option<String>,
    /// The sphere-specific avatar of this member.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sphere_avatar: Option<u64>,
    /// The sphere-specific banner of this member.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sphere_banner: Option<u64>,
    /// The sphere-specific bio of this member.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sphere_bio: Option<String>,
    /// The sphere-specific status of this member.
    #[serde(skip_serializing_if = "Option::is_none")]
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
            members: sphere.members.into_iter().map(|m| m.into()).collect(),
            emojis: sphere.emojis,
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
