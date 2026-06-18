use axum::{
    Json,
    extract::{Path, State},
};
use serde::Deserialize;

use crate::{
    application::{
        RequestHandler,
        chats::{DeleteChatMemberCommand, DeleteChatMemberCommandHandler},
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
pub async fn remove_chat_member(
    State(state): State<AppState>,
    AuthUser { claims }: AuthUser,
    Path(ChatIdUserIdParam { chat_id, user_id }): Path<ChatIdUserIdParam>,
) -> Result<(), Error> {
    let handler = DeleteChatMemberCommandHandler::new(&state);

    let command = DeleteChatMemberCommand {
        current_user_id: claims.sub,
        chat_id,
        user_id,
    };

    handler.handle(command).await
}
