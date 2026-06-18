use axum::{Json, extract::State};

use crate::{
    application::{
        RequestHandler,
        users::{GetMeQuery, GetMeQueryHandler},
    },
    error::Error,
    infra::AuthUser,
    routers::schemas::common::UserPrivateSchema,
    state::AppState,
};

#[axum::debug_handler]
pub async fn get_me(
    State(state): State<AppState>,
    AuthUser { claims }: AuthUser,
) -> Result<Json<UserPrivateSchema>, Error> {
    let handler = GetMeQueryHandler::new(&state);

    let query = GetMeQuery {
        current_user_id: claims.sub,
    };

    let result = handler.handle(query).await?;

    Ok(Json(result.into()))
}
