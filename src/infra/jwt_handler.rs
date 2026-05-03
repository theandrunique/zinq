use async_trait::async_trait;
use anyhow::Result;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
}

#[async_trait]
pub trait JwtHandler: Send + Sync {
    async fn encode(&self, user_id: &str) -> Result<String>;
    async fn decode(&self, token: &str) -> Result<Claims>;
}

pub struct JwtService {
    secret: String,
    expiration_seconds: usize,
}

impl JwtService {
    pub fn new(secret: String, expiration_seconds: usize) -> Self {
        Self {
            secret,
            expiration_seconds,
        }
    }
}

#[async_trait]
impl JwtHandler for JwtService {
    async fn encode(&self, user_id: &str) -> Result<String> {
        let now = chrono::Utc::now().timestamp() as usize;
        let claims = Claims {
            sub: user_id.to_string(),
            exp: now + self.expiration_seconds,
            iat: now,
        };

        Ok(encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )?)
    }

    async fn decode(&self, token: &str) -> Result<Claims> {
        let validation = Validation::new(Algorithm::HS256);
        let decoded = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &validation,
        )?;

        Ok(decoded.claims)
    }
}