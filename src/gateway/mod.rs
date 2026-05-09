use chrono::{DateTime, Utc};
use serde::Serialize;
use socketioxide::{SocketIo, extract::SocketRef, layer::SocketIoLayer};
use std::collections::HashMap;
use tokio::sync::broadcast;
use tracing::info;

use crate::{
    domain::{auth::User, events::DomainEvent},
    state::AppState,
};

#[derive(Serialize, Debug)]
pub struct UserPublicSchema {
    pub id: String,
    pub username: String,
    pub global_name: String,
    pub bio: Option<String>,
    pub avatar: Option<String>,
    pub timestamp: DateTime<Utc>,
}

impl UserPublicSchema {
    pub fn from(user: User) -> Self {
        Self {
            id: user.id.to_string(),
            username: user.username,
            global_name: user.display_name,
            bio: user.bio,
            avatar: user.avatar,
            timestamp: user.created_at,
        }
    }
}

#[derive(Serialize, Debug)]
pub struct HelloEvent {
    pub user_id: String,
    pub session_id: String,
}

async fn socket_event_loop(io: SocketIo, mut events: broadcast::Receiver<DomainEvent>) {
    while let Ok(event) = events.recv().await {
        match event {
            DomainEvent::UserCreate { user } => {
                io.broadcast()
                    .emit("user:create", &UserPublicSchema::from(user))
                    .await
                    .ok();
            }
            DomainEvent::ChatCreate { chat: _chat } => {}
            DomainEvent::ChatMemberAdded { .. } => {}
            DomainEvent::ChatMemberRemoved { .. } => {}
            DomainEvent::MessageCreated { .. } => {}
            DomainEvent::MessageUpdated { .. } => {}
        }
    }
}

pub fn gateway(app_state: AppState) -> SocketIoLayer {
    let state = app_state.clone();
    let (layer, io) = SocketIo::builder().with_state(app_state).build_layer();

    io.ns("/", move |socket: SocketRef| async move {
        info!("Socket.IO connected: {:?} {:?}", socket.ns(), socket.id);

        let uri = &socket.req_parts().uri;
        let query = uri.query().unwrap_or_default();

        info!("Full URI: {}", uri);

        let token = query
            .split('&')
            .find(|s| s.starts_with("access_token="))
            .and_then(|s| s.strip_prefix("access_token="))
            .map(|s| s.to_string());

        info!("Extracted token: {:?}", token);

        let token = match token {
            Some(t) => t,
            None => {
                socket.emit("error", "Missing access_token").ok();
                socket.disconnect().ok();
                return;
            }
        };

        info!("Token (first 50 chars): {}", &token[..50.min(token.len())]);

        let claims = match state.jwt_handler.verify_access_token(&token) {
            Ok(c) => c,
            Err(e) => {
                info!("Token verification error: {}", e);
                socket.emit("error", &e.to_string()).ok();
                socket.disconnect().ok();
                return;
            }
        };

        info!(
            "Token verified successfully, user_id: {}, session_id: {}",
            claims.sub, claims.session_id
        );

        let hello = HelloEvent {
            user_id: claims.sub.clone(),
            session_id: claims.session_id.clone(),
        };

        info!(
            "Sending hello: user_id={}, session_id={}",
            hello.user_id, hello.session_id
        );
        socket.emit("hello", &hello).ok();
    });

    tokio::spawn(socket_event_loop(io.clone(), state.event_bus.subscribe()));

    return layer;
}
