use axum::{
    Router,
    routing::{delete, get, post, put},
};

use crate::state::AppState;

mod add_chat_member;
mod create_cloud_attachment;
mod create_message;
mod edit_message;
mod get_chat;
mod get_chat_attachments;
mod get_chat_messages;
mod remove_chat_member;

use add_chat_member::add_chat_member;
use create_cloud_attachment::create_cloud_attachment;
use create_message::create_message;
use edit_message::edit_message;
use get_chat::get_chat;
use get_chat_attachments::get_chat_attachments;
use get_chat_messages::get_chat_messages;
use remove_chat_member::remove_chat_member;

pub fn chat_router(state: AppState) -> Router {
    Router::new()
        .route("/{chat_id}", get(get_chat))
        .route("/{chat_id}/attachments", get(get_chat_attachments))
        .route("/{chat_id}/attachments", post(create_cloud_attachment))
        .route("/{chat_id}/messages", get(get_chat_messages))
        .route("/{chat_id}/messages", post(create_message))
        .route("/{chat_id}/messages/{message_id}", put(edit_message))
        .route("/{chat_id}/members/{member_id}", put(add_chat_member))
        .route("/{chat_id}/members/{member_id}", delete(remove_chat_member))
        .with_state(state)
}
