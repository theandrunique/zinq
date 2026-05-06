use std::collections::HashMap;

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
    InvalidRequestBody(HashMap<String, Vec<String>>),
    InternalServerError(anyhow::Error),
}

impl From<anyhow::Error> for Error {
    fn from(inner: anyhow::Error) -> Self {
        Error::InternalServerError(inner)
    }
}
