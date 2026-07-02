use std::collections::HashMap;

use axum::{
    Json,
    extract::{Path, State},
};
use serde::Deserialize;
use serde_qs::web::QsQuery;
use serde_with::{DisplayFromStr, serde_as};

use crate::{
    application::{
        RequestHandler,
        messages::{CountMessagesQuery, CountMessagesQueryHandler, CountMessagesQueryResponse},
    },
    error::Error,
    infra::AuthUser,
    state::AppState,
};

#[serde_as]
#[derive(Deserialize)]
pub struct GetMessagesCountQueryParams {
    #[serde_as(as = "Option<DisplayFromStr>")]
    from: Option<i64>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    to: Option<i64>,
}

#[derive(Deserialize)]
pub struct ChatIdParam {
    chat_id: i64,
}

#[axum::debug_handler]
pub async fn get_messages_count(
    State(state): State<AppState>,
    AuthUser { claims }: AuthUser,
    Path(ChatIdParam { chat_id }): Path<ChatIdParam>,
    QsQuery(params): QsQuery<GetMessagesCountQueryParams>,
) -> Result<Json<CountMessagesQueryResponse>, Error> {
    let handler = CountMessagesQueryHandler::new(&state);

    let command = CountMessagesQuery {
        current_user_id: claims.sub,
        chat_id: chat_id,
        from_message_id: params.from.unwrap_or(i64::MIN),
        to_message_id: params.to.unwrap_or(i64::MAX),
    };

    let result = handler.handle(command).await?;

    Ok(Json(result))
}
