use std::collections::HashMap;

use crate::domain::chats::ChatPermissions;

#[derive(Debug)]
pub enum Error {
    AuthInvalidCredentials,
    AuthInvalidToken,
    AuthTotpRequired {
        username: String,
        global_name: String,
    },
    UsernameAlreadyInUse,
    EmailAlreadyInUse,
    UserNotFound(i64),
    UsersNotFound(Vec<i64>),
    UserNotMember {
        user_id: i64,
        chat_id: i64,
    },
    ChatNotFound(i64),
    ChatTypeNotSupported {
        chat_id: i64,
    },
    InsufficientPermissions {
        permission: ChatPermissions,
        chat_id: i64,
    },
    UserAlreadyMember {
        user_id: i64,
        chat_id: i64,
    },
    InvalidRequestBody(HashMap<String, Vec<String>>),
    InternalServerError(anyhow::Error),
}

impl From<anyhow::Error> for Error {
    fn from(inner: anyhow::Error) -> Self {
        Error::InternalServerError(inner)
    }
}
