use crate::{
    error::{ErrorKind, Result, ResultExt},
    websocket::Post,
};
use chrono::prelude::{DateTime, Utc};
use log::debug;
use reqwest::{Client as WebClient, StatusCode};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fmt, str::FromStr};
use url::Url;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Client {
    base_url: Url,
    token: String,
}

impl Client {
    pub fn new<B, T>(base_url: B, token: T) -> Result<Client>
    where
        B: AsRef<str>,
        T: Into<String>,
    {
        Ok(Client {
            base_url: Url::parse(base_url.as_ref())?,
            token: token.into(),
        })
    }

    pub fn is_token_valid(&self) -> bool {
        self.get_users(0, 0).is_ok()
    }

    pub fn get_users(&self, page: usize, per_page: usize) -> Result<Vec<User>> {
        let client = WebClient::new();
        let mut url = self.base_url.join("/api/v4/users")?;
        url.query_pairs_mut()
            .append_pair("page", &page.to_string())
            .append_pair("per_page", &per_page.to_string());
        let mut res = client
            .get(url)
            .header("authorization", format!("bearer {}", self.token))
            .send()
            .chain_err(|| "Failed to send webrequest")?;

        match res.status() {
            // 400
            StatusCode::BAD_REQUEST => Err(ErrorKind::InvalidOrMissingParameter.into()),
            // 401
            StatusCode::UNAUTHORIZED => Err(ErrorKind::MissingAccessToken.into()),
            // 403
            StatusCode::FORBIDDEN => Err(ErrorKind::MissingPermissions.into()),
            // 200
            // StatusCode::Ok => Ok(res.json()?),
            _ => Ok(res.json()?),
        }
    }

    pub fn get_users_by_id(&self, ids: &[String]) -> Result<Vec<User>> {
        let client = WebClient::new();
        let url = self.base_url.join("/api/v4/users/ids")?;
        let mut res = client
            .post(url)
            .header("authorization", format!("bearer {}", self.token))
            .json(&ids)
            .send()
            .chain_err(|| "Failed to send webrequest")?;

        match res.status() {
            // 400
            StatusCode::BAD_REQUEST => Err(ErrorKind::InvalidOrMissingParameter.into()),
            // 401
            StatusCode::UNAUTHORIZED => Err(ErrorKind::MissingAccessToken.into()),
            // 403
            StatusCode::FORBIDDEN => Err(ErrorKind::MissingPermissions.into()),
            // 200
            // StatusCode::Ok => Ok(res.json()?),
            _ => Ok(res.json()?),
        }
    }

    pub fn get_channel_by_id<S>(&self, id: S) -> Result<Channel>
    where
        S: AsRef<str>,
    {
        let client = WebClient::new();
        let url = self.base_url.join("/api/v4/channels/")?.join(id.as_ref())?;
        let mut res = client
            .get(url)
            .header("authorization", format!("bearer {}", self.token))
            .send()
            .chain_err(|| "Failed to send webrequest")?;
        debug!("get_channel_by_id response {}", res.status());

        match res.status() {
            // 400
            StatusCode::BAD_REQUEST => Err(ErrorKind::InvalidOrMissingParameter.into()),
            // 401
            StatusCode::UNAUTHORIZED => Err(ErrorKind::MissingAccessToken.into()),
            // 403
            StatusCode::FORBIDDEN => Err(ErrorKind::MissingPermissions.into()),
            // 200
            // StatusCode::Ok => Ok(res.json()?),
            _ => Ok(res.json()?),
        }
    }

    pub fn create_post(&self, post: &CreatePostRequest) -> Result<Post> {
        let client = WebClient::new();
        let url = self.base_url.join("/api/v4/posts")?;
        let mut res = client
            .post(url)
            .header("authorization", format!("bearer {}", self.token))
            .json(&post)
            .send()
            .chain_err(|| "Failed to send webrequest")?;
        debug!("create_post response {}", res.status());

        match res.status() {
            // 400
            StatusCode::BAD_REQUEST => Err(ErrorKind::InvalidOrMissingParameter.into()),
            // 401
            StatusCode::UNAUTHORIZED => Err(ErrorKind::MissingAccessToken.into()),
            // 403
            StatusCode::FORBIDDEN => Err(ErrorKind::MissingPermissions.into()),
            // 200
            // StatusCode::Ok => Ok(res.json()?),
            _ => Ok(res.json()?),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct User {
    pub id: String,
    #[serde(with = "crate::serialize::ts_seconds")]
    pub create_at: DateTime<Utc>,
    #[serde(with = "crate::serialize::ts_seconds")]
    pub update_at: DateTime<Utc>,
    #[serde(with = "crate::serialize::ts_seconds")]
    pub delete_at: DateTime<Utc>,
    pub username: String,
    pub first_name: String,
    pub last_name: String,
    pub nickname: String,
    pub email: String,
    pub email_verified: Option<bool>,
    pub auth_data: String,
    pub auth_service: String,
    pub position: String,
    #[serde(with = "::serde_with::rust::StringWithSeparator::<::serde_with::SpaceSeparator>")]
    pub roles: HashSet<UserRole>,
    // pub roles: UserRole,
    pub locale: String,
    // pub notify_props: {},
    // pub props: {},
    #[serde(
        skip_serializing_if = "Option::is_none",
        with = "crate::serialize::option_ts_milliseconds",
        default
    )]
    pub last_password_update: Option<DateTime<Utc>>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        with = "crate::serialize::option_ts_milliseconds",
        default
    )]
    pub last_picture_update: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failed_attempts: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mfa_active: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<Timezone>,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Timezone {
    automatic_timezone: String,
    manual_timezone: String,
    #[serde(with = "::serde_with::rust::display_fromstr")]
    use_automatic_timezone: bool,
}

#[derive(Debug, Eq, Hash, PartialEq, Copy, Clone, Ord, PartialOrd)]
pub enum UserRole {
    SystemUser,
    SystemAdmin,
    ChannelUser,
}

impl fmt::Display for UserRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            UserRole::SystemUser => write!(f, "system_user"),
            UserRole::SystemAdmin => write!(f, "system_admin"),
            UserRole::ChannelUser => write!(f, "channel_user"),
        }
    }
}

impl FromStr for UserRole {
    type Err = String;

    fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
        match s {
            "system_user" => Ok(UserRole::SystemUser),
            "system_admin" => Ok(UserRole::SystemAdmin),
            "channel_user" => Ok(UserRole::ChannelUser),
            _ => Err(format!(
                "Unexpected value '{}', expected one of 'system_user', 'system_admin'",
                s
            )),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Ord, PartialOrd, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct Channel {
    id: String,
    #[serde(with = "crate::serialize::ts_seconds")]
    pub create_at: DateTime<Utc>,
    #[serde(with = "crate::serialize::ts_seconds")]
    pub update_at: DateTime<Utc>,
    #[serde(with = "crate::serialize::ts_seconds")]
    pub delete_at: DateTime<Utc>,
    pub team_id: String,
    #[serde(rename = "type")]
    pub type_: ChannelType,
    pub display_name: String,
    pub header: String,
    #[serde(with = "crate::serialize::ts_seconds")]
    pub last_post_at: DateTime<Utc>,
    pub total_msg_count: u64,
    #[serde(with = "crate::serialize::ts_seconds")]
    pub extra_update_at: DateTime<Utc>,
    pub creator_id: String,
}

#[derive(Debug, Deserialize, Serialize, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ChannelType {
    #[serde(rename = "O")]
    Open,
    #[serde(rename = "P")]
    Private,
    #[serde(rename = "D")]
    DirectMessage,
    #[serde(rename = "G")]
    Group,
    #[serde(rename = "I")]
    Internal,
}

#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct CreatePostRequest {
    pub channel_id: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub root_id: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub file_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub props: Option<String>,
}
