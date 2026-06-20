use std::collections::HashMap;

use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::Deserialize;
use serde_qs::web::QsQuery;
use serde_with::{DisplayFromStr, Map, serde_as};

use crate::{
    application::{
        RequestHandler,
        messages::{GetLastMessagesQuery, GetLastMessagesQueryHandler, GetMessagesQuery, GetMessagesQueryHandler},
    },
    error::Error,
    infra::AuthUser,
    routers::schemas::common::MessageSchema,
    state::AppState,
};

#[serde_as]
#[derive(Deserialize)]
pub struct GetLastMessagesQueryParams {
    #[serde_as(as = "Vec<DisplayFromStr>")]
    chat_ids: Vec<i64>,
}

#[axum::debug_handler]
pub async fn get_last_messages(
    State(state): State<AppState>,
    AuthUser { claims }: AuthUser,
    QsQuery(params): QsQuery<GetLastMessagesQueryParams>,
) -> Result<Json<HashMap<String, MessageSchema>>, Error> {
    let handler = GetLastMessagesQueryHandler::new(&state);

    let command = GetLastMessagesQuery {
        current_user_id: claims.sub,
        chat_ids: params.chat_ids,
    };

    let result = handler.handle(command).await?;

    let mut response: HashMap<String, MessageSchema> = HashMap::new();
    for message in result {
        response.insert(message.chat_id.to_string(), message.into());
    }

    Ok(Json(response))
}
