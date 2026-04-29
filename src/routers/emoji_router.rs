use axum::{Json, Router, extract::State, routing::get};

use crate::domain::emojis::default_emoji::build_default_manifest;
use crate::state::AppState;

async fn get_manifest(State(_state): State<AppState>) -> Json<serde_json::Value> {
    let manifest = build_default_manifest();
    Json(serde_json::to_value(manifest).unwrap())
}

pub fn emoji_router(state: AppState) -> Router {
    Router::new()
        .route("/manifest.json", get(get_manifest))
        .with_state(state)
}