use axum::{
    Router,
    routing::{delete, get, post, put},
};

use crate::state::AppState;

mod ack_messages;
mod add_chat_member;
mod create_cloud_attachment;
mod create_message;
mod edit_message;
mod get_chat;
mod get_chat_attachments;
mod get_chat_messages;
mod get_last_messages;
mod get_message_acks;
mod get_messages_count;
mod remove_chat_member;

use ack_messages::ack_messages;
use add_chat_member::add_chat_member;
use create_cloud_attachment::create_cloud_attachment;
use create_message::create_message;
use edit_message::edit_message;
use get_chat::get_chat;
use get_chat_attachments::get_chat_attachments;
use get_chat_messages::get_chat_messages;
use get_last_messages::get_last_messages;
use get_message_acks::get_message_acks;
use get_messages_count::get_messages_count;
use remove_chat_member::remove_chat_member;

pub fn chat_router(state: AppState) -> Router {
    Router::new()
        .route("/{chat_id}", get(get_chat))
        .route("/{chat_id}/attachments", get(get_chat_attachments))
        .route("/{chat_id}/attachments", post(create_cloud_attachment))
        .route("/{chat_id}/messages", get(get_chat_messages))
        .route("/messages", get(get_last_messages))
        .route("/{chat_id}/messages", post(create_message))
        .route("/{chat_id}/messages/acks", put(ack_messages))
        .route("/{chat_id}/messages/count", get(get_messages_count))
        .route("/{chat_id}/messages/{message_id}", put(edit_message))
        .route(
            "/{chat_id}/messages/{message_id}/acks",
            get(get_message_acks),
        )
        .route("/{chat_id}/members/{user_id}", put(add_chat_member))
        .route("/{chat_id}/members/{user_id}", delete(remove_chat_member))
        .with_state(state)
}
