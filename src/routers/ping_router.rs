use axum::{Json, Router, routing::get};
use serde::{Serialize};

use crate::error::Error;

pub fn ping_router() -> Router {
    Router::new()
        .route("/", get(ping_handler))
}

#[derive(Serialize)]
pub struct PingResponse {
    status: String,
}

async fn ping_handler() -> Result<Json<PingResponse>, Error> {
    Ok(Json(PingResponse { status: "ok".to_string() }))
}
