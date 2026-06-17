use axum::http::Uri;
use serde::Serialize;
use socketioxide::{
    SocketIo,
    extract::{Extension, SocketRef, State},
    handler::connect::ConnectHandler,
    layer::SocketIoLayer,
};
use std::fmt;

use crate::{infra::TokenClaims, state::AppState};

#[derive(Serialize, Debug)]
pub struct HelloEvent {
    pub user_id: String,
    pub session_id: String,
}

#[derive(Debug)]
pub struct AuthError(pub String);

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for AuthError {}

fn parse_token(uri: &Uri) -> Option<String> {
    let query = uri.query().unwrap_or_default();

    query
        .split('&')
        .find(|s| s.starts_with("access_token=") || s.starts_with("accessToken="))
        .and_then(|s| {
            s.strip_prefix("access_token=")
                .or(s.strip_prefix("accessToken="))
        })
        .map(|s| s.to_string())
}

async fn auth_middleware(
    socket: SocketRef,
    State(app_state): State<AppState>,
) -> Result<(), AuthError> {
    let uri = socket.req_parts().uri.clone();
    let access_token = parse_token(&uri);

    let token = match access_token {
        Some(t) => t,
        None => {
            tracing::info!("Missing access_token query parameter");
            return Err(AuthError(
                "Missing access_token query parameter".to_string(),
            ));
        }
    };

    tracing::info!("Token: {}...", &token[..15.min(token.len())]);

    let claims = match app_state.jwt_handler.verify_access_token(&token) {
        Ok(c) => c,
        Err(e) => {
            tracing::info!("Token verification error: {}", e);
            return Err(AuthError(format!("Token verification error: {}", e)));
        }
    };

    tracing::info!(
        "Token verified successfully, user_id: {}, session_id: {}",
        claims.sub,
        claims.session_id
    );

    socket.extensions.insert(claims);

    Ok(())
}

async fn connect_handler(socket: SocketRef, Extension(claims): Extension<TokenClaims>) {
    tracing::info!("Socket.IO connected: {:?} {:?}", socket.ns(), socket.id);

    let user_id = claims.sub.to_string();
    socket.join(format!("user:{}", &user_id));

    let hello = HelloEvent {
        user_id: user_id.clone(),
        session_id: claims.session_id.to_string(),
    };

    tracing::info!(
        "Sending hello: user_id={}, session_id={}",
        hello.user_id,
        hello.session_id
    );

    if let Err(e) = socket.emit("hello", &hello) {
        tracing::warn!("Failed to emit hello: {}", e);
    }
}

pub fn gateway(app_state: AppState) -> (SocketIoLayer, SocketIo) {
    let (layer, io) = SocketIo::builder().with_state(app_state).build_layer();

    io.ns("/", connect_handler.with(auth_middleware));

    (layer, io)
}
