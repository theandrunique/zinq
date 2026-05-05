use anyhow::Result;
use base32::Alphabet;
use totp_rs::{Algorithm, Secret, TOTP};

const ISSUER: &str = "Zinq";
const DEFAULT_ACCOUNT_NAME: &str = "unknown";

pub trait TotpHandler: Send + Sync {
    fn generate_secret(&self) -> Result<String>;
    fn generate_code(&self, secret: &str) -> Result<String>;
    fn verify_code(&self, secret: &str, code: &str) -> Result<bool>;
    fn generate_qr_url(&self, secret: &str, account_name: &str) -> Result<String>;
}

pub struct TotpService;

impl TotpService {
    pub fn new() -> Self {
        Self
    }

    fn build_totp(&self, secret: &str, account_name: Option<&str>) -> Result<TOTP> {
        let secret_bytes = base32::decode(Alphabet::Rfc4648 { padding: false }, secret)
            .ok_or_else(|| anyhow::anyhow!("Invalid secret encoding"))?;

        TOTP::new(
            Algorithm::SHA256,
            6,
            1,
            30,
            secret_bytes,
            Some(ISSUER.to_string()),
            account_name.unwrap_or(DEFAULT_ACCOUNT_NAME).to_string(),
        )
        .map_err(anyhow::Error::from)
    }
}

impl TotpHandler for TotpService {
    fn generate_secret(&self) -> Result<String> {
        let secret = Secret::generate_secret();
        Ok(secret.to_encoded().to_string())
    }

    fn generate_code(&self, secret: &str) -> Result<String> {
        let totp = self.build_totp(secret, None)?;
        let timestamp = chrono::Utc::now().timestamp() as u64;
        Ok(totp.generate(timestamp))
    }

    fn verify_code(&self, secret: &str, code: &str) -> Result<bool> {
        let totp = self.build_totp(secret, None)?;
        let timestamp = chrono::Utc::now().timestamp() as u64;
        Ok(totp.check(code, timestamp))
    }

    fn generate_qr_url(&self, secret: &str, account_name: &str) -> Result<String> {
        let totp = self.build_totp(secret, Some(account_name))?;
        Ok(totp.get_url())
    }
}
