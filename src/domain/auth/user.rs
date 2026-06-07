use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SessionLifetime {
    Week,
    Month,
    Month3,
    Month6,
    Month12,
}

impl std::fmt::Display for SessionLifetime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            SessionLifetime::Week => "WEEK",
            SessionLifetime::Month => "MONTH",
            SessionLifetime::Month3 => "MONTH3",
            SessionLifetime::Month6 => "MONTH6",
            SessionLifetime::Month12 => "MONTH12",
        };
        write!(f, "{}", str)
    }
}

impl FromStr for SessionLifetime {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "WEEK" => Ok(SessionLifetime::Week),
            "MONTH" => Ok(SessionLifetime::Month),
            "MONTH3" => Ok(SessionLifetime::Month3),
            "MONTH6" => Ok(SessionLifetime::Month6),
            "MONTH12" => Ok(SessionLifetime::Month12),
            _ => Err(format!("Unknown SessionLifetime: {}", s)),
        }
    }
}

#[derive(Clone, Debug)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub username_updated_at: DateTime<Utc>,
    pub password_hash: String,
    pub password_updated_at: DateTime<Utc>,
    pub avatar: Option<String>,
    pub sessions_lifetime: SessionLifetime,
    pub bio: Option<String>,
    pub display_name: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub totp_key: Option<Vec<u8>>,
    pub mfa: bool,
    pub email: String,
    pub is_email_verified: bool,
    pub email_updated_at: DateTime<Utc>,
}

pub struct UserCreateRequest {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub display_name: String,
    pub email: String,
}

impl User {
    pub fn create(request: UserCreateRequest) -> Self {
        let current_time = Utc::now();

        Self {
            id: request.id,
            username: request.username,
            username_updated_at: current_time,
            password_hash: request.password_hash,
            password_updated_at: current_time,
            avatar: None,
            sessions_lifetime: SessionLifetime::Month3,
            bio: None,
            display_name: request.display_name,
            is_active: true,
            created_at: current_time,
            totp_key: None,
            mfa: false,
            email: request.email,
            is_email_verified: false,
            email_updated_at: current_time,
        }
    }
}
