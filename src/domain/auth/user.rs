use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SessionLifetime {
    WEEK,
    MONTH,
    MONTH3,
    MONTH6,
    MONTH12,
}

impl std::fmt::Display for SessionLifetime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            SessionLifetime::WEEK => "WEEK",
            SessionLifetime::MONTH => "MONTH",
            SessionLifetime::MONTH3 => "MONTH3",
            SessionLifetime::MONTH6 => "MONTH6",
            SessionLifetime::MONTH12 => "MONTH12",
        };
        write!(f, "{}", str)
    }
}

impl FromStr for SessionLifetime {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "WEEK" => Ok(SessionLifetime::WEEK),
            "MONTH" => Ok(SessionLifetime::MONTH),
            "MONTH3" => Ok(SessionLifetime::MONTH3),
            "MONTH6" => Ok(SessionLifetime::MONTH6),
            "MONTH12" => Ok(SessionLifetime::MONTH12),
            _ => Err(format!("Unknown SessionLifetime: {}", s)),
        }
    }
}

#[derive(Clone, Debug)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub username_updated_timestamp: DateTime<Utc>,
    pub password_hash: String,
    pub password_updated_timestamp: DateTime<Utc>,
    pub avatar: Option<String>,
    pub sessions_lifetime: SessionLifetime,
    pub bio: Option<String>,
    pub global_name: String,
    pub is_active: bool,
    pub timestamp: DateTime<Utc>,
    pub totp_key: Option<Vec<u8>>,
    pub mfa: bool,
    pub email: String,
    pub is_email_verified: bool,
    pub email_updated_timestamp: DateTime<Utc>,
}

impl User {
    pub fn create(
        id: i64,
        username: String,
        password_hash: String,
        global_name: String,
        email: String,
    ) -> Self {
        let timestamp = Utc::now();

        Self {
            id: id,
            username: username,
            username_updated_timestamp: timestamp,
            password_hash,
            password_updated_timestamp: timestamp,
            avatar: None,
            sessions_lifetime: SessionLifetime::MONTH3,
            bio: None,
            global_name: global_name,
            is_active: true,
            timestamp: timestamp,
            totp_key: None,
            mfa: false,
            email: email,
            is_email_verified: false,
            email_updated_timestamp: timestamp,
        }
    }
}
