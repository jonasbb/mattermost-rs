use serialize;

use chrono::prelude::*;
use std::collections::HashMap;
use api::{Channel, ChannelType, User};

#[derive(Debug, Serialize)]
pub struct Message {
    pub event: Events,
    pub broadcast: Broadcast,
    pub seq: usize,
}

impl<'de> ::serde::de::Deserialize<'de> for Message {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::de::Deserializer<'de>,
    {
        use serde::de::*;

        struct HExpected;

        impl Expected for HExpected {
            fn fmt(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                formatter.write_str("struct `Message`")
            }
        }

        #[derive(Default, Deserialize)]
        struct Helper {
            broadcast: Option<Broadcast>,
            seq: Option<usize>,
        }

        let mut helper = Helper::default();

        use serde_json::Value;
        let mut value: Value = Value::deserialize(deserializer)?;
        if let Some(ref mut map) = value.as_object_mut() {
            if let Some(v) = map.remove("broadcast") {
                helper.broadcast = ::serde_json::from_value(v).ok();
            } else {
                // TODO make better error message here
                return Err(Error::invalid_value(Unexpected::StructVariant, &HExpected));
            }
            if let Some(v) = map.remove("seq") {
                helper.seq = ::serde_json::from_value(v).ok();
            } else {
                // TODO make better error message here
                return Err(Error::invalid_value(Unexpected::StructVariant, &HExpected));
            }
        } else {
            // TODO make better error message here
            return Err(Error::invalid_value(Unexpected::StructVariant, &HExpected));
        }

        let event: Events = ::serde_json::from_value(value).map_err(Error::custom)?;

        if helper.broadcast.is_none() || helper.seq.is_none() {
            // TODO make better error message here
            return Err(Error::invalid_value(Unexpected::StructVariant, &HExpected));
        }

        Ok(Message {
            event: event,
            broadcast: helper.broadcast.unwrap(),
            seq: helper.seq.unwrap(),
        })
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "event", content = "data", deny_unknown_fields, rename_all = "snake_case")]
pub enum Events {
    Hello { server_version: String },
    StatusChange { status: Status, user_id: String },
    EphemeralMessage {
        #[serde(deserialize_with = "::serialize::deserialize_embedded_json")]
        post: Post,
    },
    Typing { parent_id: String, user_id: String },
    Posted {
        channel_display_name: String,
        channel_name: String,
        channel_type: ChannelType,
        #[serde(deserialize_with = "::serialize::deserialize_embedded_json")]
        post: Post,
        sender_name: String,
        team_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none",
                deserialize_with = "::serialize::deserialize_embedded_json")]
        mentions: Option<Vec<String>>,
        // TODO this might also be a boolean
        image: Option<String>,
        // TODO this might also be a boolean
        #[serde(rename = "otherFile")]
        other_file: Option<String>,
    },
    ReactionAdded {
        #[serde(deserialize_with = "::serialize::deserialize_embedded_json")]
        reaction: Reaction,
    },
    PostEdited {
        #[serde(deserialize_with = "::serialize::deserialize_embedded_json")]
        post: Post,
    },
    ChannelCreated { channel_id: String, team_id: String },
    PreferencesChanged { preferences: String },
    UserUpdated { user: User },
    PostDeleted {
        #[serde(deserialize_with = "::serialize::deserialize_embedded_json")]
        post: Post,
    },
    ChannelViewed { channel_id: String },
    PreferencesDeleted { preferences: String },
    ChannelUpdated {
        #[serde(deserialize_with = "::serialize::deserialize_embedded_json")]
        channel: Channel,
    },
    ReactionRemoved {
        #[serde(deserialize_with = "::serialize::deserialize_embedded_json")]
        reaction: Reaction,
    },
    NewUser { user_id: String },
    EmojiAdded {
        #[serde(deserialize_with = "::serialize::deserialize_embedded_json")]
        emoji: Emoji,
    },
    ChannelDeleted { channel_id: String },
    DirectAdded { teammate_id: String },
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Broadcast {
    pub omit_users: Option<HashMap<String, bool>>,
    pub user_id: String,
    pub channel_id: String,
    pub team_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Status {
    Online,
    Away,
    Offline,
}

#[derive(Debug, Deserialize, Serialize)]
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
    pub root_id: String,
    pub parent_id: String,
    pub original_id: String,
    pub message: String,
    #[serde(rename = "type")]
    pub type_: PostType,
    pub props: PostProps,
    // FIXME hashtags are a whitespace-separated string
    pub hashtags: String,
    pub pending_post_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_reactions: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum PostType {
    #[serde(rename = "")]
    UserMessage,
    SystemEphemeral,
    SystemJoinChannel,
    SystemHeaderChange,
    SystemChannelDeleted,
}

#[derive(Debug, Deserialize, Serialize)]
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
}

#[derive(Debug, Deserialize, Serialize)]
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

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Reaction {
    pub user_id: String,
    pub post_id: String,
    pub emoji_name: String,
    #[serde(with = "serialize::ts_seconds")]
    pub create_at: DateTime<Utc>,
}
