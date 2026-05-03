use async_trait::async_trait;
use anyhow::Result;
use base32::Alphabet;
use totp_rs::{Algorithm, Secret, TOTP};

const ISSUER: &str = "Zinq";

#[async_trait]
pub trait TotpHandler: Send + Sync {
    async fn generate_secret(&self) -> Result<String>;
    async fn generate_code(&self, secret: &str) -> Result<String>;
    async fn verify_code(&self, secret: &str, code: &str) -> Result<bool>;
    async fn generate_qr_url(&self, secret: &str, account_name: &str) -> Result<String>;
}

pub struct TotpService;

impl TotpService {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl TotpHandler for TotpService {
    async fn generate_secret(&self) -> Result<String> {
        let secret = Secret::generate_secret();
        Ok(secret.to_encoded().to_string())
    }

    async fn generate_code(&self, secret: &str) -> Result<String> {
        let secret_bytes = base32::decode(Alphabet::Rfc4648 { padding: false }, secret)
            .ok_or_else(|| anyhow::anyhow!("Invalid secret encoding"))?;

        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            secret_bytes,
            None,
            ISSUER.to_string(),
        )?;

        let timestamp = chrono::Utc::now().timestamp() as u64;
        Ok(totp.generate(timestamp))
    }

    async fn verify_code(&self, secret: &str, code: &str) -> Result<bool> {
        let secret_bytes = base32::decode(Alphabet::Rfc4648 { padding: false }, secret)
            .ok_or_else(|| anyhow::anyhow!("Invalid secret encoding"))?;

        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            secret_bytes,
            None,
            ISSUER.to_string(),
        )?;

        let timestamp = chrono::Utc::now().timestamp() as u64;
        Ok(totp.check(code, timestamp))
    }

    async fn generate_qr_url(&self, secret: &str, account_name: &str) -> Result<String> {
        let url = format!(
            "otpauth://totp/{}:{}?secret={}&issuer={}&algorithm=SHA1&digits=6&period=30",
            ISSUER,
            urlencoding::encode(account_name),
            secret,
            urlencoding::encode(ISSUER)
        );
        Ok(url)
    }
}