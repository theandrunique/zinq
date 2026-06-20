use axum::{
    Json,
    extract::{Query, State},
    routing::get,
};
use serde::Deserialize;

use crate::{
    application::{
        RequestHandler,
        event_logs::{GetEventLogsQuery, GetEventLogsQueryHandler},
    },
    error::Error,
    infra::AuthUser,
    routers::schemas::common::EventLogSchema,
    state::AppState,
};

pub fn sync_router(state: AppState) -> axum::Router {
    axum::Router::new()
        .route("/sync", get(sync_handler))
        .with_state(state)
}

#[derive(Deserialize)]
pub struct GetMessagesQueryParams {
    after: Option<i64>,
    limit: Option<i32>,
}

async fn sync_handler(
    State(state): State<AppState>,
    AuthUser { claims }: AuthUser,
    Query(params): Query<GetMessagesQueryParams>,
) -> Result<Json<Vec<EventLogSchema>>, Error> {
    let handler = GetEventLogsQueryHandler::new(&state);

    let query = GetEventLogsQuery {
        current_user_id: claims.sub,
        after: params.after,
        limit: params.limit.unwrap_or(50),
    };

    let result = handler.handle(query).await?;

    Ok(Json(result.into_iter().map(|log| log.into()).collect()))
}
