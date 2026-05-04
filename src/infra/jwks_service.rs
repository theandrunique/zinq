use anyhow::{Context, Result};
use pem::parse;
use rsa::RsaPrivateKey;
use rsa::pkcs8::DecodePrivateKey;
use rsa::traits::PublicKeyParts;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Clone)]
pub struct KeyPair {
    id: String,
    private_key: RsaPrivateKey,
    private_key_pem: String,
}

impl KeyPair {
    pub fn private_key_pem(&self) -> &[u8] {
        self.private_key_pem.as_bytes()
    }
}

pub struct JwksServiceImpl {
    active_key: KeyPair,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Jwks {
    pub keys: Vec<Jwk>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Jwk {
    pub kty: String,
    #[serde(rename = "use")]
    pub use_: String,
    pub kid: String,
    pub alg: String,
    pub n: String,
    pub e: String,
}

impl JwksServiceImpl {
    pub fn new(keys_directory: &str) -> Result<Self> {
        let key_path = Path::new(keys_directory).join("key.pem");
        let key_data = fs::read(&key_path)
            .with_context(|| format!("Failed to read key file: {}", key_path.display()))?;

        let pem_block = parse(&key_data).with_context(|| "Failed to parse PEM file")?;

        let private_key = RsaPrivateKey::from_pkcs8_der(pem_block.contents())
            .with_context(|| "Failed to decode PKCS8 private key")?;

        let key_pem = String::from_utf8_lossy(&key_data).to_string();
        let key_id = "1".to_string();

        Ok(Self {
            active_key: KeyPair {
                id: key_id,
                private_key,
                private_key_pem: key_pem,
            },
        })
    }

    pub fn get_active_key(&self) -> Result<&KeyPair> {
        Ok(&self.active_key)
    }

    pub fn get_jwks(&self) -> Jwks {
        let key = &self.active_key.private_key;
        let public_key = key.to_public_key();

        let n_bytes = public_key.n().to_bytes_be();
        let e_bytes = public_key.e().to_bytes_be();

        let n = base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, &n_bytes);
        let e = base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, &e_bytes);

        Jwks {
            keys: vec![Jwk {
                kty: "RSA".to_string(),
                use_: "sig".to_string(),
                kid: self.active_key.id.clone(),
                alg: "RS256".to_string(),
                n,
                e,
            }],
        }
    }
}

pub trait JwksService: Send + Sync {
    fn get_active_key(&self) -> Result<&KeyPair>;
    fn get_jwks(&self) -> Jwks;
}

impl JwksService for JwksServiceImpl {
    fn get_active_key(&self) -> Result<&KeyPair> {
        Ok(&self.active_key)
    }

    fn get_jwks(&self) -> Jwks {
        let key = &self.active_key.private_key;
        let public_key = key.to_public_key();

        let n_bytes = public_key.n().to_bytes_be();
        let e_bytes = public_key.e().to_bytes_be();

        let n = base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, &n_bytes);
        let e = base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, &e_bytes);

        Jwks {
            keys: vec![Jwk {
                kty: "RSA".to_string(),
                use_: "sig".to_string(),
                kid: self.active_key.id.clone(),
                alg: "RS256".to_string(),
                n,
                e,
            }],
        }
    }
}
