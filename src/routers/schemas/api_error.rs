use std::collections::HashMap;

use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};

use crate::error::Error;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum ErrorCode {
    AuthInvalidCredentials,
    AuthInvalidToken,
    AuthTotpRequired,
    InvalidJson,
    InvalidRequestBody,
    UsernameAlreadyInUse,
    UserNotFound,
    UsersNotFound,
    UserNotMember,
    ChatNotFound,
    AttachmentInvalidUploadFilename,
    AttachmentObjectNotFound,
    ChatTypeNotSupported,
    InsufficientPermissions,
    UserAlreadyMember,
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
            ErrorCode::AuthInvalidToken => StatusCode::UNAUTHORIZED,
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
            Error::AuthInvalidToken => ApiError::new(ErrorCode::AuthInvalidToken, "Invalid token"),
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
            Error::UserNotFound(user_id) => ApiError::new(
                ErrorCode::UserNotFound,
                format!("User not found: {}", user_id),
            ),
            Error::UsersNotFound(user_ids) => ApiError::new(
                ErrorCode::UsersNotFound,
                format!("Users not found: {:?}", user_ids),
            ),
            Error::UserNotMember { user_id, chat_id } => ApiError::new(
                ErrorCode::UserNotMember,
                format!("User {} is not a member of chat {}", user_id, chat_id),
            ),
            Error::ChatNotFound(chat_id) => ApiError::new(
                ErrorCode::ChatNotFound,
                format!("Chat not found: {}", chat_id),
            ),
            Error::ChatTypeNotSupported { chat_id: _ } => ApiError::new(
                ErrorCode::ChatTypeNotSupported,
                "Only group chats support this operation",
            ),
            Error::InsufficientPermissions {
                permission,
                chat_id,
            } => ApiError::new(
                ErrorCode::InsufficientPermissions,
                format!(
                    "Requires '{}' permission for chat '{}'",
                    permission, chat_id
                ),
            ),
            Error::AttachmentObjectNotFound { upload_filename } => ApiError::new(
                ErrorCode::AttachmentObjectNotFound,
                format!("Attachment '{}' not found in object storage", upload_filename),
            ),
            Error::AttachmentInvalidUploadFilename { upload_filename } => ApiError::new(
                ErrorCode::AttachmentInvalidUploadFilename,
                format!("Invalid upload filename format '{}'", upload_filename),
            ),
            Error::UserAlreadyMember { user_id, chat_id } => ApiError::new(
                ErrorCode::UserAlreadyMember,
                format!("User {} is already a member of chat {}", user_id, chat_id),
            ),
            Error::InternalServerError(e) => {
                tracing::error!(error = ?e, "Unhandled domain error");
                ApiError::new(ErrorCode::InternalServerError, "Internal server error")
            }
            Error::InvalidRequestBody(e) => ApiError::validation(e),
        };

        return e.into_response();
    }
}
