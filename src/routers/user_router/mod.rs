use axum::{
    Router,
    routing::{get, post},
};

mod create_chat;
mod get_dm_chat;
mod get_me;
mod get_user_chats;

use create_chat::create_chat;
use get_dm_chat::get_dm_chat;
use get_me::get_me;
use get_user_chats::get_user_chats;

use crate::state::AppState;

pub fn user_router(state: AppState) -> Router {
    Router::new()
        .route("/@me", get(get_me))
        .route("/@me/chats", get(get_user_chats))
        .route("/@me/chats", post(create_chat))
        .route("/@me/dms/{user_id}", get(get_dm_chat))
        .with_state(state)
}
