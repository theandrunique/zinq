use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
pub enum SessionLifetime {
    WEEK,
    MONTH,
    MONTH3,
    MONTH6,
    MONTH12,
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
        password: String,
        global_name: String,
        email: String,
    ) -> Self {
        let timestamp = Utc::now();

        Self {
            id: id,
            username: username,
            username_updated_timestamp: timestamp,
            password_hash: password,
            password_updated_timestamp: timestamp,
            avatar: None,
            sessions_lifetime: SessionLifetime::MONTH6,
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
