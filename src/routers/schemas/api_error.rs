use std::collections::HashMap;

use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};

use crate::error::Error;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum ErrorCode {
    AuthInvalidCredentials,
    AuthTotpRequired,
    InvalidJson,
    InvalidRequestBody,
    UsernameAlreadyInUse,
    EmailAlreadyInUse,
    InternalServerError,
}

#[derive(Serialize, Deserialize, Debug)]
struct ApiError {
    code: ErrorCode,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    errors: Option<HashMap<String, Vec<String>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<HashMap<String, String>>,
}

impl ApiError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            errors: None,
            metadata: None,
        }
    }

    pub fn validation(errors: HashMap<String, Vec<String>>) -> Self {
        Self {
            code: ErrorCode::InvalidRequestBody,
            message: "Invalid request body".to_string(),
            errors: Some(errors),
            metadata: None,
        }
    }

    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let status = match self.code {
            ErrorCode::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            _ => StatusCode::BAD_REQUEST,
        };

        return (status, Json(self)).into_response();
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let e = match self {
            Error::AuthInvalidCredentials => {
                ApiError::new(ErrorCode::AuthInvalidCredentials, "Invalid Credentials")
            }
            Error::AuthTotpRequired {
                username,
                global_name,
            } => ApiError::new(ErrorCode::AuthTotpRequired, "TOTP required").with_metadata(
                HashMap::from([
                    ("username".to_string(), username),
                    ("global_name".to_string(), global_name),
                ]),
            ),
            Error::UsernameAlreadyInUse => {
                ApiError::new(ErrorCode::UsernameAlreadyInUse, "Username already in use")
            }
            Error::EmailAlreadyInUse => {
                ApiError::new(ErrorCode::EmailAlreadyInUse, "Email already in use")
            }
            Error::InternalServerError(e) => {
                tracing::error!(error = ?e, "Unhandled domain error");
                ApiError::new(ErrorCode::InternalServerError, "Internal server error")
            }
            Error::InvalidRequestBody(e) => ApiError::validation(e),
        };

        return e.into_response();
    }
}
