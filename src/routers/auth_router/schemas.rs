use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct TokensResponseSchema {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
}
