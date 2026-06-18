use chrono::{DateTime, Utc};

pub enum VerificationCodeScenario {
    VerifyEmail,
}

pub struct VerificationCode {
    pub id: String,
    pub scenario: VerificationCodeScenario,
    pub code_hash: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub attempts: i32,
}

impl VerificationCode {
    pub fn new(
        id: String,
        scenario: VerificationCodeScenario,
        code_hash: String,
        expires_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            scenario,
            code_hash,
            created_at: Utc::now(),
            expires_at,
            attempts: 0,
        }
    }
}
