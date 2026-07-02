use axum::{
    Json,
    extract::{Path, State},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    application::{
        RequestHandler,
        messages::{GetMessageAcksQuery, GetMessageAcksQueryHandler},
    },
    error::Error,
    infra::AuthUser,
    state::AppState,
};

#[derive(Deserialize)]
pub struct ChatIdMessageIdParam {
    chat_id: i64,
    message_id: i64,
}

#[derive(Serialize)]
pub struct MessageAckSchema {
    pub user_id: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct MessageAcksResponse {
    pub acks: Vec<MessageAckSchema>,
}

#[axum::debug_handler]
pub async fn get_message_acks(
    State(state): State<AppState>,
    AuthUser { claims }: AuthUser,
    Path(ChatIdMessageIdParam { chat_id, message_id }): Path<ChatIdMessageIdParam>,
) -> Result<Json<MessageAcksResponse>, Error> {
    let handler = GetMessageAcksQueryHandler::new(&state);

    let result = handler
        .handle(GetMessageAcksQuery {
            current_user_id: claims.sub,
            chat_id,
            message_id,
        })
        .await?;

    Ok(Json(MessageAcksResponse {
        acks: result
            .into_iter()
            .map(|a| MessageAckSchema {
                user_id: a.user_id.to_string(),
                timestamp: a.created_at,
            })
            .collect(),
    }))
}
