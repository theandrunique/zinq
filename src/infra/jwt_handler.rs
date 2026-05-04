use anyhow::Result;
use async_trait::async_trait;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

use super::jwks_service::{JwksService, KeyPair};

const TOKEN_TYPE_ACCESS: &str = "access";
const TOKEN_TYPE_REFRESH: &str = "refresh";

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessClaims {
    pub sub: String,
    pub session_id: String,
    #[serde(rename = "token_type")]
    pub token_type: String,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshClaims {
    pub sub: String,
    pub session_id: String,
    #[serde(rename = "token_type")]
    pub token_type: String,
    pub exp: usize,
    pub iat: usize,
}

#[async_trait]
pub trait JwtHandler: Send + Sync {
    async fn generate_access_token(&self, user_id: &str, session_id: &str) -> Result<String>;
    async fn generate_refresh_token(
        &self,
        user_id: &str,
        session_id: &str,
        expires_in_seconds: usize,
    ) -> Result<String>;
    async fn verify_access_token(&self, token: &str) -> Result<AccessClaims>;
    async fn verify_refresh_token(&self, token: &str) -> Result<RefreshClaims>;
}

pub struct JwtService {
    jwks_service: Box<dyn JwksService>,
    access_token_expiration_seconds: usize,
}

impl JwtService {
    pub fn new(jwks_service: Box<dyn JwksService>, access_token_expiration_seconds: usize) -> Self {
        Self {
            jwks_service,
            access_token_expiration_seconds,
        }
    }

    fn encode_token(
        &self,
        key_pair: &KeyPair,
        claims: &(impl Serialize + serde::de::DeserializeOwned),
        expiration_seconds: usize,
    ) -> Result<String> {
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(key_pair.id().to_string());
        let encoding_key = EncodingKey::from_rsa_pem(key_pair.private_key_pem())?;

        encode(&header, claims, &encoding_key).map_err(anyhow::Error::from)
    }

    fn decode_token<T: for<'de> Deserialize<'de>>(
        &self,
        key_pair: &KeyPair,
        token: &str,
    ) -> Result<T> {
        let validation = Validation::new(Algorithm::RS256);
        let decoding_key = DecodingKey::from_rsa_pem(key_pair.private_key_pem())?;

        let decoded = decode::<T>(token, &decoding_key, &validation)?;

        Ok(decoded.claims)
    }
}

#[async_trait]
impl JwtHandler for JwtService {
    async fn generate_access_token(&self, user_id: &str, session_id: &str) -> Result<String> {
        let key_pair = self.jwks_service.get_active_key()?;

        let now = chrono::Utc::now().timestamp() as usize;
        let exp = now + self.access_token_expiration_seconds;

        let claims = AccessClaims {
            sub: user_id.to_string(),
            session_id: session_id.to_string(),
            token_type: TOKEN_TYPE_ACCESS.to_string(),
            exp,
            iat: now,
        };

        self.encode_token(key_pair, &claims, self.access_token_expiration_seconds)
    }

    async fn generate_refresh_token(
        &self,
        user_id: &str,
        session_id: &str,
        expires_in_seconds: usize,
    ) -> Result<String> {
        let key_pair = self.jwks_service.get_active_key()?;

        let now = chrono::Utc::now().timestamp() as usize;
        let exp = now + expires_in_seconds;

        let claims = RefreshClaims {
            sub: user_id.to_string(),
            session_id: session_id.to_string(),
            token_type: TOKEN_TYPE_REFRESH.to_string(),
            exp,
            iat: now,
        };

        self.encode_token(key_pair, &claims, expires_in_seconds)
    }

    async fn verify_access_token(&self, token: &str) -> Result<AccessClaims> {
        let key_pair = self.jwks_service.get_active_key()?;

        let claims: AccessClaims = self.decode_token(key_pair, token)?;

        if claims.token_type != TOKEN_TYPE_ACCESS {
            anyhow::bail!("Invalid token type: expected access");
        }

        Ok(claims)
    }

    async fn verify_refresh_token(&self, token: &str) -> Result<RefreshClaims> {
        let key_pair = self.jwks_service.get_active_key()?;

        let claims: RefreshClaims = self.decode_token(key_pair, token)?;

        if claims.token_type != TOKEN_TYPE_REFRESH {
            anyhow::bail!("Invalid token type: expected refresh");
        }

        Ok(claims)
    }
}
