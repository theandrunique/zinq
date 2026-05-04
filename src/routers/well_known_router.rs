use axum::{Json, extract::State, routing::get};

use crate::infra::jwks_service::Jwks;
use crate::state::AppState;

pub fn well_known_router(state: AppState) -> axum::Router {
    axum::Router::new()
        .route("/jwks.json", get(jwks_handler))
        .with_state(state)
}

async fn jwks_handler(State(state): State<AppState>) -> Json<Jwks> {
    Json(state.jwks_service.get_jwks())
}
