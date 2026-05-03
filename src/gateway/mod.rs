use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use socketioxide::{
    SocketIo,
    extract::{Data, SocketRef},
    layer::SocketIoLayer,
};
use tokio::sync::broadcast;
use tracing::info;

use crate::{
    domain::{auth::User, events::DomainEvent},
    state::AppState,
};

#[derive(Deserialize, Serialize, Debug)]
struct AuthData {
    access_token: String,
}

#[derive(Serialize)]
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

async fn socket_event_loop(io: SocketIo, mut events: broadcast::Receiver<DomainEvent>) {
    while let Ok(event) = events.recv().await {
        match event {
            DomainEvent::UserCreate { user } => {
                io.broadcast()
                    .emit("user:create", &UserPublicSchema::from(user))
                    .await
                    .ok();
            }
        }
    }
}

async fn on_connect(socket: SocketRef) {
    info!("Socket.IO connected: {:?} {:?}", socket.ns(), socket.id);

    socket.on("auth", async |socket: SocketRef, Data::<AuthData>(data)| {
        info!(?data, "Received event:");
        socket.emit("message-back", &data).ok();
    })
}

pub fn gateway(app_state: AppState) -> SocketIoLayer {
    let (layer, io) = SocketIo::builder().build_layer();

    io.ns("/", on_connect);

    tokio::spawn(socket_event_loop(
        io.clone(),
        app_state.event_bus.subscribe(),
    ));

    return layer;
}
