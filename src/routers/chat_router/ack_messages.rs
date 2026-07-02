use axum::{
    Json,
    extract::{Path, State},
};
use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::{
    application::{
        RequestHandler,
        messages::{MessageAckCommand, MessageAckCommandHandler, MessageAckInput},
    },
    error::Error,
    infra::AuthUser,
    state::AppState,
};

#[derive(Deserialize)]
pub struct AckMessagesRequest {
    pub last_read_message_id: i64,
    #[serde(default)]
    pub acks: Vec<AckInputRequest>,
}

#[derive(Deserialize)]
pub struct AckInputRequest {
    pub message_id: i64,
    pub acked_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct ChatIdParam {
    chat_id: i64,
}

#[axum::debug_handler]
pub async fn ack_messages(
    State(state): State<AppState>,
    AuthUser { claims }: AuthUser,
    Path(ChatIdParam { chat_id }): Path<ChatIdParam>,
    Json(body): Json<AckMessagesRequest>,
) -> Result<(), Error> {
    let handler = MessageAckCommandHandler::new(&state);

    let command = MessageAckCommand {
        current_user_id: claims.sub,
        chat_id,
        last_read_message_id: body.last_read_message_id,
        acks: body
            .acks
            .into_iter()
            .map(|a| MessageAckInput {
                message_id: a.message_id,
                acked_at: a.acked_at,
            })
            .collect(),
    };

    handler.handle(command).await?;

    Ok(())
}
