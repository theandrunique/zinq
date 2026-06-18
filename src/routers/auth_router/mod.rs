use crate::state::AppState;
use axum::{
    Router,
    routing::{get, post},
};

mod login;
mod refresh_token;
mod register;
mod schemas;

use login::login;
use refresh_token::refresh_token;
use register::register;

pub fn auth_router(state: AppState) -> Router {
    Router::new()
        .route("/sign-up", post(register))
        .route("/sign-in", post(login))
        .route("/refresh", post(refresh_token))
        .with_state(state)
}
