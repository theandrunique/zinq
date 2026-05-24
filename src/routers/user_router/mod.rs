use axum::{
    Router,
    routing::{get, post},
};

mod create_chat;
mod get_user_chats;
mod get_dm_chat;

use create_chat::create_chat;
use get_user_chats::get_user_chats;
use get_dm_chat::get_dm_chat;

use crate::state::AppState;

pub fn user_router(state: AppState) -> Router {
    Router::new()
        .route("/@me/chats", get(get_user_chats))
        .route("/@me/chats", post(create_chat))
        .route("/@me/dms/{user_id}", get(get_dm_chat))
        .with_state(state)
}
