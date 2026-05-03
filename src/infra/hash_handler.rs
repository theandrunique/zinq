use async_trait::async_trait;
use anyhow::Result;
use bcrypt::{hash, verify, DEFAULT_COST};
use tokio::task;

#[async_trait]
pub trait HashHandler: Send + Sync {
    async fn hash(&self, password: &str) -> Result<String>;
    async fn verify(&self, password: &str, hash: &str) -> Result<bool>;
}

pub struct BcryptHandler;

impl BcryptHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl HashHandler for BcryptHandler {
    async fn hash(&self, password: &str) -> Result<String> {
        let password = password.to_string();
        task::spawn_blocking(move || {
            hash(password, DEFAULT_COST).map_err(anyhow::Error::from)
        })
        .await?
    }

    async fn verify(&self, password: &str, hash: &str) -> Result<bool> {
        let password = password.to_string();
        let hash = hash.to_string();
        task::spawn_blocking(move || {
            verify(password, &hash).map_err(anyhow::Error::from)
        })
        .await?
    }
}