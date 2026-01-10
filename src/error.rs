use std::collections::HashMap;

pub enum Error {
    AuthInvalidCredentials,
    AuthTotpRequired {
        username: String,
        global_name: String,
    },
    InvalidRequestBody(HashMap<String, Vec<String>>),
}
