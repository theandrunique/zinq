use axum::{
    Router,
    routing::{get, post},
};

mod create_chat;
mod get_user_chats;

use create_chat::create_chat;
use get_user_chats::get_user_chats;

use crate::state::AppState;

pub fn user_router(state: AppState) -> Router {
    Router::new()
        .route("/@me/chats", get(get_user_chats))
        .route("/@me/chats", post(create_chat))
        .with_state(state)
}
