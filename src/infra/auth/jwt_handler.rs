use anyhow::Result;
use jsonwebtoken::{Algorithm, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};
use uuid::Uuid;

use crate::infra::auth::jwks_service::{JwksService, KeyPair};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TokenType {
    Access,
    Refresh,
}

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    #[serde_as(as = "DisplayFromStr")]
    pub sub: i64,
    #[serde_as(as = "DisplayFromStr")]
    pub session_id: Uuid,
    #[serde(rename = "token_type")]
    pub token_type: TokenType,
    pub exp: i64,
    pub iat: i64,
}

pub trait JwtHandler: Send + Sync {
    fn generate_access_token(&self, user_id: i64, session_id: Uuid) -> Result<String>;
    fn generate_refresh_token(
        &self,
        user_id: i64,
        session_id: Uuid,
        expires_in_seconds: i64,
    ) -> Result<String>;
    fn verify_access_token(&self, token: &str) -> Result<TokenClaims>;
    fn verify_refresh_token(&self, token: &str) -> Result<TokenClaims>;
}

pub struct JwtService<J: JwksService> {
    jwks_service: J,
    access_token_expiration_seconds: i64,
}

impl<J: JwksService> JwtService<J> {
    pub fn new(jwks_service: J, access_token_expiration_seconds: i64) -> Self {
        Self {
            jwks_service,
            access_token_expiration_seconds,
        }
    }

    fn encode_token(&self, key_pair: &KeyPair, claims: &TokenClaims) -> Result<String> {
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(key_pair.kid().to_string());

        encode(&header, claims, key_pair.encoding_key()).map_err(anyhow::Error::from)
    }

    fn verify_token(
        &self,
        key_pair: &KeyPair,
        token: &str,
        expected_type: TokenType,
    ) -> Result<TokenClaims> {
        let validation = Validation::new(Algorithm::RS256);

        let decoded = decode::<TokenClaims>(token, key_pair.decoding_key(), &validation)?;

        let claims = decoded.claims;

        if claims.token_type != expected_type {
            anyhow::bail!("Invalid token type: expected {:?}", expected_type);
        }

        Ok(claims)
    }
}

impl<J: JwksService> JwtHandler for JwtService<J> {
    fn generate_access_token(&self, user_id: i64, session_id: Uuid) -> Result<String> {
        let key_pair = self.jwks_service.get_active_key();

        let now = chrono::Utc::now().timestamp();
        let exp = now + self.access_token_expiration_seconds;

        let claims = TokenClaims {
            sub: user_id,
            session_id: session_id,
            token_type: TokenType::Access,
            exp,
            iat: now,
        };

        self.encode_token(key_pair, &claims)
    }

    fn generate_refresh_token(
        &self,
        user_id: i64,
        session_id: Uuid,
        expires_in_seconds: i64,
    ) -> Result<String> {
        let key_pair = self.jwks_service.get_active_key();

        let now = chrono::Utc::now().timestamp();
        let exp = now + expires_in_seconds;

        let claims = TokenClaims {
            sub: user_id,
            session_id: session_id,
            token_type: TokenType::Refresh,
            exp,
            iat: now,
        };

        self.encode_token(key_pair, &claims)
    }

    fn verify_access_token(&self, token: &str) -> Result<TokenClaims> {
        let key_pair = self.jwks_service.get_active_key();
        self.verify_token(key_pair, token, TokenType::Access)
    }

    fn verify_refresh_token(&self, token: &str) -> Result<TokenClaims> {
        let key_pair = self.jwks_service.get_active_key();
        self.verify_token(key_pair, token, TokenType::Refresh)
    }
}
