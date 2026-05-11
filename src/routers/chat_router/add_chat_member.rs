use axum::{
    Json,
    extract::{Path, State},
};
use serde::Deserialize;

use crate::{
    application::{
        RequestHandler,
        chats::{AddChatMemberCommand, AddChatMemberCommandHandler},
    },
    error::Error,
    infra::AuthUser,
    state::AppState,
};

#[derive(Deserialize)]
pub struct ChatIdUserIdParam {
    chat_id: i64,
    user_id: i64,
}

#[axum::debug_handler]
pub async fn add_chat_member(
    State(state): State<AppState>,
    AuthUser { claims }: AuthUser,
    Path(ChatIdUserIdParam { chat_id, user_id }): Path<ChatIdUserIdParam>,
) -> Result<(), Error> {
    let handler = AddChatMemberCommandHandler::new(&state);

    let command = AddChatMemberCommand {
        current_user_id: claims.sub,
        chat_id: chat_id,
        user_id: user_id,
    };

    handler.handle(command).await
}
