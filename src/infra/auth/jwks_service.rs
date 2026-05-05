use anyhow::{Context, Result};
use jsonwebtoken::{DecodingKey, EncodingKey};
use pem::parse;
use rsa::{
    RsaPrivateKey, RsaPublicKey,
    pkcs8::{DecodePrivateKey, EncodePublicKey},
    traits::PublicKeyParts,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

#[derive(Clone)]
pub struct KeyPair {
    kid: String,
    public_key: RsaPublicKey,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl KeyPair {
    pub fn kid(&self) -> &str {
        &self.kid
    }

    pub fn encoding_key(&self) -> &EncodingKey {
        &self.encoding_key
    }

    pub fn decoding_key(&self) -> &DecodingKey {
        &self.decoding_key
    }

    pub fn public_key(&self) -> &RsaPublicKey {
        &self.public_key
    }
}

pub trait JwksService: Send + Sync {
    fn get_active_key(&self) -> &KeyPair;
    fn get_jwks(&self) -> Jwks;
}

pub struct FileJwksService {
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

impl From<&KeyPair> for Jwk {
    fn from(key_pair: &KeyPair) -> Self {
        let public_key = key_pair.public_key();
        let n_bytes = public_key.n().to_bytes_be();
        let e_bytes = public_key.e().to_bytes_be();

        let n = base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, &n_bytes);
        let e = base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, &e_bytes);

        Jwk {
            kty: "RSA".to_string(),
            use_: "sig".to_string(),
            kid: key_pair.kid().to_string(),
            alg: "RS256".to_string(),
            n,
            e,
        }
    }
}

fn compute_jwk_thumbprint(n: &[u8], e: &[u8]) -> String {
    let n_b64 = base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, n);
    let e_b64 = base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, e);

    let jwk_json = serde_json::json!({"e": e_b64, "kty": "RSA", "n": n_b64});

    let canonical = serde_json::to_string(&jwk_json).expect("Failed to serialize JWK");
    let mut hasher = Sha256::new();
    hasher.update(canonical.as_bytes());
    let result = hasher.finalize();

    base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, &result)
}

impl FileJwksService {
    pub fn load_from_directory(keys_directory: &str) -> Result<Self> {
        let key_path = Path::new(keys_directory).join("key.pem");
        let key_data = fs::read(&key_path)
            .with_context(|| format!("Failed to read key file: {}", key_path.display()))?;

        let pem_block = parse(&key_data).with_context(|| "Failed to parse PEM file")?;

        let private_key = RsaPrivateKey::from_pkcs8_der(pem_block.contents())
            .with_context(|| "Failed to decode PKCS8 private key")?;

        let public_key: RsaPublicKey = private_key.to_public_key();
        let n_bytes = public_key.n().to_bytes_be();
        let e_bytes = public_key.e().to_bytes_be();

        let kid = compute_jwk_thumbprint(&n_bytes, &e_bytes);

        let encoding_key = EncodingKey::from_rsa_pem(&key_data)
            .context("Failed to create encoding key from PEM")?;

        let public_key_der = public_key
            .to_public_key_der()
            .context("Failed to encode public key DER")?;
        let public_key_pem = format!(
            "-----BEGIN PUBLIC KEY-----\n{}\n-----END PUBLIC KEY-----",
            base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                public_key_der.as_bytes()
            )
        );
        let decoding_key = DecodingKey::from_rsa_pem(public_key_pem.as_bytes())
            .context("Failed to create decoding key from PEM")?;

        Ok(Self {
            active_key: KeyPair {
                kid,
                public_key,
                encoding_key,
                decoding_key,
            },
        })
    }
}

impl JwksService for FileJwksService {
    fn get_active_key(&self) -> &KeyPair {
        &self.active_key
    }

    fn get_jwks(&self) -> Jwks {
        Jwks {
            keys: vec![Jwk::from(&self.active_key)],
        }
    }
}
