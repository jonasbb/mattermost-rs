use crate::{
    api::{Channel, ChannelType, User, UserRole},
    serialize,
};
use chrono::prelude::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Message {
    Push(MessagePush),
    Reply(MessageReply),
}

#[derive(Debug, Serialize, Deserialize)]
// #[serde(untagged)]
pub struct MessagePush {
    #[serde(flatten)]
    pub event: Events,
    pub broadcast: Broadcast,
    pub seq: usize,
}

#[derive(Debug, Serialize, Deserialize)]
// #[serde(untagged)]
pub struct MessageReply {
    pub status: MessageStatus,
    pub seq_reply: usize,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE", deny_unknown_fields)]
pub enum MessageStatus {
    Ok,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq)]
#[serde(
    tag = "event",
    content = "data",
    deny_unknown_fields,
    rename_all = "snake_case"
)]
pub enum Events {
    Hello {
        server_version: String,
    },
    StatusChange {
        status: Status,
        user_id: String,
    },
    EphemeralMessage {
        #[serde(with = "::serde_with::json::nested")]
        post: Post,
    },
    Typing {
        parent_id: String,
        user_id: String,
    },
    Posted {
        channel_display_name: String,
        channel_name: String,
        channel_type: ChannelType,
        #[serde(with = "::serde_with::json::nested")]
        post: Post,
        sender_name: String,
        team_id: String,
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            with = "::serde_with::json::nested"
        )]
        mentions: Option<Vec<String>>,
        // TODO this might also be a boolean
        image: Option<String>,
        // TODO this might also be a boolean
        #[serde(rename = "otherFile")]
        other_file: Option<String>,
    },
    ReactionAdded {
        #[serde(with = "::serde_with::json::nested")]
        reaction: Reaction,
    },
    PostEdited {
        #[serde(with = "::serde_with::json::nested")]
        post: Post,
    },
    ChannelCreated {
        channel_id: String,
        team_id: String,
    },
    PreferencesChanged {
        preferences: String,
    },
    UserUpdated {
        user: User,
    },
    PostDeleted {
        #[serde(with = "::serde_with::json::nested")]
        post: Post,
    },
    ChannelViewed {
        channel_id: String,
    },
    PreferencesDeleted {
        preferences: String,
    },
    ChannelUpdated {
        #[serde(with = "::serde_with::json::nested")]
        channel: Channel,
    },
    ReactionRemoved {
        #[serde(with = "::serde_with::json::nested")]
        reaction: Reaction,
    },
    NewUser {
        user_id: String,
    },
    EmojiAdded {
        #[serde(with = "::serde_with::json::nested")]
        emoji: Emoji,
    },
    ChannelDeleted {
        channel_id: String,
        #[serde(with = "serialize::option_ts_milliseconds", default)]
        delete_at: Option<DateTime<Utc>>,
    },
    DirectAdded {
        teammate_id: String,
    },
    UpdateTeam {
        #[serde(with = "::serde_with::json::nested")]
        team: Team,
    },
    UserAdded {
        team_id: String,
        user_id: String,
    },
    UserRemoved {
        remover_id: String,
        user_id: String,
    },
    LeaveTeam {
        team_id: String,
        user_id: String,
    },
    ConfigChanged {
        config: Config,
    },
    GroupAdded {
        #[serde(with = "::serde_with::json::nested")]
        teammate_ids: Vec<String>,
    },
    DeleteTeam {
        #[serde(with = "::serde_with::json::nested")]
        team: Team,
    },
    ChannelMemberUpdated {
        #[serde(rename = "channelMember", with = "::serde_with::json::nested")]
        channel_member: ChannelMember,
    },
}

#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Broadcast {
    pub omit_users: Option<HashMap<String, bool>>,
    pub user_id: String,
    pub channel_id: String,
    pub team_id: String,
}

#[derive(Debug, Deserialize, Serialize, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Status {
    Online,
    Away,
    #[serde(rename = "dnd")]
    DoNotDisturb,
    Offline,
}

#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Post {
    pub id: String,
    #[serde(with = "serialize::ts_seconds")]
    pub create_at: DateTime<Utc>,
    #[serde(with = "serialize::ts_seconds")]
    pub update_at: DateTime<Utc>,
    #[serde(with = "serialize::ts_seconds")]
    pub edit_at: DateTime<Utc>,
    #[serde(with = "serialize::ts_seconds")]
    pub delete_at: DateTime<Utc>,
    pub is_pinned: bool,
    pub user_id: String,
    pub channel_id: String,
    // TODO empty equals not set
    pub root_id: String,
    pub parent_id: String,
    pub original_id: String,
    pub message: String,
    #[serde(rename = "type")]
    pub type_: PostType,
    pub props: PostProps,
    #[serde(with = "::serde_with::rust::StringWithSeparator::<::serde_with::SpaceSeparator>")]
    pub hashtags: HashSet<String>,
    pub pending_post_id: String,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub file_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_reactions: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum PostType {
    #[serde(rename = "")]
    UserMessage,
    SystemEphemeral,
    SystemJoinChannel,
    SystemHeaderChange,
    SystemChannelDeleted,
    SystemPurposeChange,
    SystemDisplaynameChange,
    SystemAddToChannel,
    SystemRemoveFromChannel,
    SystemJoinTeam,
    SystemRemoveFromTeam,
    SystemLeaveChannel,
}

#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PostProps {
    #[serde(skip_serializing_if = "Option::is_none")]
    override_icon_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    old_header: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    new_header: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    new_purpose: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    old_purpose: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    new_displayname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    old_displayname: Option<String>,
    #[serde(rename = "addedUsername", skip_serializing_if = "Option::is_none")]
    added_username: Option<String>,
    #[serde(rename = "removedUsername", skip_serializing_if = "Option::is_none")]
    removed_username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    add_channel_member: Option<AddChannelMember>,
    #[serde(skip_serializing_if = "Option::is_none")]
    from_webhook: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    override_username: Option<String>,
    #[serde(rename = "addedUserId", skip_serializing_if = "Option::is_none")]
    added_user_id: Option<String>,
    #[serde(rename = "userId", skip_serializing_if = "Option::is_none")]
    user_id: Option<String>,
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    channel_mentions: HashMap<String, ChannelInfo>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[serde(deny_unknown_fields)]
pub struct AddChannelMember {
    pub post_id: String,
    pub user_ids: Vec<String>,
    pub usernames: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[serde(deny_unknown_fields)]
pub struct Emoji {
    pub id: String,
    #[serde(with = "serialize::ts_seconds")]
    pub create_at: DateTime<Utc>,
    #[serde(with = "serialize::ts_seconds")]
    pub update_at: DateTime<Utc>,
    #[serde(with = "serialize::ts_seconds")]
    pub delete_at: DateTime<Utc>,
    pub creator_id: String,
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[serde(deny_unknown_fields)]
pub struct Reaction {
    pub user_id: String,
    pub post_id: String,
    pub emoji_name: String,
    #[serde(with = "serialize::ts_seconds")]
    pub create_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[serde(deny_unknown_fields)]
pub struct Team {
    pub id: String,
    #[serde(with = "serialize::ts_seconds")]
    pub create_at: DateTime<Utc>,
    #[serde(with = "serialize::ts_seconds")]
    pub update_at: DateTime<Utc>,
    #[serde(with = "serialize::ts_seconds")]
    pub delete_at: DateTime<Utc>,
    pub display_name: String,
    pub name: String,
    pub description: String,
    pub email: String,
    #[serde(rename = "type")]
    pub type_: ChannelType,
    pub company_name: String,
    pub allowed_domains: String,
    pub invite_id: String,
    pub allow_open_invite: bool,
    #[serde(default)]
    pub scheme_id: Option<String>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        with = "serialize::option_ts_milliseconds",
        default
    )]
    pub last_team_icon_update: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Config(pub BTreeMap<String, String>);

#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ChannelMember {
    pub channel_id: String,
    pub user_id: String,
    #[serde(with = "::serde_with::rust::StringWithSeparator::<::serde_with::SpaceSeparator>")]
    pub roles: HashSet<UserRole>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        with = "serialize::option_ts_milliseconds",
        default
    )]
    pub last_viewed_at: Option<DateTime<Utc>>,
    pub msg_count: u32,
    pub mention_count: u32,
    pub notify_props: NotifyProps,
    #[serde(
        skip_serializing_if = "Option::is_none",
        with = "serialize::option_ts_milliseconds",
        default
    )]
    pub last_update_at: Option<DateTime<Utc>>,
    pub scheme_user: bool,
    pub scheme_admin: bool,
    #[serde(with = "::serde_with::rust::StringWithSeparator::<::serde_with::SpaceSeparator>")]
    pub explicit_roles: HashSet<UserRole>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[serde(deny_unknown_fields)]
pub struct NotifyProps {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub desktop: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore_channel_mentions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mark_unread: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub push: Option<String>,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChannelInfo {
    pub display_name: String,
}
