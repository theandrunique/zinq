use chrono::{DateTime, Utc};

pub enum VerificationCodeScenario {
    VerifyEmail,
}

pub struct VerificationCode {
    pub id: String,
    pub scenario: VerificationCodeScenario,
    pub code_hash: String,
    pub timestamp: DateTime<Utc>,
    pub expires_timestamp: DateTime<Utc>,
    pub attempts: i32,
}

impl VerificationCode {
    pub fn new(
        id: String,
        scenario: VerificationCodeScenario,
        code_hash: String,
        expires_timpestamp: DateTime<Utc>,
    ) -> Self {
        Self {
            id: id,
            scenario: scenario,
            code_hash: code_hash,
            timestamp: Utc::now(),
            expires_timestamp: expires_timpestamp,
            attempts: 0,
        }
    }
}
