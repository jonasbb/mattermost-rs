use chrono::prelude::*;
use error::*;
use reqwest::{Client as WebClient, StatusCode};
use reqwest::header::{Authorization, Bearer};
use std::collections::HashSet;
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
        url
            .query_pairs_mut()
            .append_pair("page", &page.to_string())
            .append_pair("per_page", &per_page.to_string());
        let mut res = client
            .get(url)
            .header(Authorization(Bearer { token: self.token.clone() }))
            .send()
            .chain_err(|| "Failed to send webrequest")?;

        match res.status() {
            // 400
            StatusCode::BadRequest => Err(ErrorKind::InvalidOrMissingParameter.into()),
            // 401
            StatusCode::Unauthorized => Err(ErrorKind::MissingAccessToken.into()),
            // 403
            StatusCode::Forbidden => Err(ErrorKind::MissingPermissions.into()),
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
            .header(Authorization(Bearer { token: self.token.clone() }))
            .json(&ids)
            .send()
            .chain_err(|| "Failed to send webrequest")?;

        match res.status() {
            // 400
            StatusCode::BadRequest => Err(ErrorKind::InvalidOrMissingParameter.into()),
            // 401
            StatusCode::Unauthorized => Err(ErrorKind::MissingAccessToken.into()),
            // 403
            StatusCode::Forbidden => Err(ErrorKind::MissingPermissions.into()),
            // 200
            // StatusCode::Ok => Ok(res.json()?),
            _ => Ok(res.json()?),
        }
    }

    pub fn get_channel_by_id(&self, id: String) -> Result<Channel> {
        let client = WebClient::new();
        let url = self.base_url.join("/api/v4/channels/")?.join(&id.to_string())?;
        let mut res = client
            .get(url)
            .header(Authorization(Bearer { token: self.token.clone() }))
            .send()
            .chain_err(|| "Failed to send webrequest")?;

        match res.status() {
            // 400
            StatusCode::BadRequest => Err(ErrorKind::InvalidOrMissingParameter.into()),
            // 401
            StatusCode::Unauthorized => Err(ErrorKind::MissingAccessToken.into()),
            // 403
            StatusCode::Forbidden => Err(ErrorKind::MissingPermissions.into()),
            // 200
            // StatusCode::Ok => Ok(res.json()?),
            _ => Ok(res.json()?),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct User {
    pub id: String,
    #[serde(with = "::serialize::ts_seconds")]
    pub create_at: DateTime<Utc>,
    #[serde(with = "::serialize::ts_seconds")]
    pub update_at: DateTime<Utc>,
    #[serde(with = "::serialize::ts_seconds")]
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
    #[serde(deserialize_with = "::serialize::deserialize_string_set")]
    pub roles: HashSet<UserRole>,
    // pub roles: UserRole,
    pub locale: String,
    // pub notify_props: {},
    // pub props: {},
    #[serde(skip_serializing_if = "Option::is_none", with = "::serialize::option_ts_milliseconds",
            default)]
    pub last_password_update: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none", with = "::serialize::option_ts_milliseconds",
            default)]
    pub last_picture_update: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failed_attempts: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mfa_active: Option<bool>,
}

#[derive(Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    SystemUser,
    SystemAdmin,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Channel {
    id: String,
    #[serde(with = "::serialize::ts_seconds")]
    pub create_at: DateTime<Utc>,
    #[serde(with = "::serialize::ts_seconds")]
    pub update_at: DateTime<Utc>,
    #[serde(with = "::serialize::ts_seconds")]
    pub delete_at: DateTime<Utc>,
    pub team_id: String,
    #[serde(rename = "type")]
    pub type_: ChannelType,
    pub display_name: String,
    pub header: String,
    #[serde(with = "::serialize::ts_seconds")]
    pub last_post_at: DateTime<Utc>,
    pub total_msg_count: u64,
    #[serde(with = "::serialize::ts_seconds")]
    pub extra_update_at: DateTime<Utc>,
    pub creator_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum ChannelType {
    #[serde(rename = "O")]
    Open,
    #[serde(rename = "P")]
    Private,
    #[serde(rename = "D")]
    DirectMessage,
}
