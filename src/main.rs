use std::net::SocketAddr;

use axum::Router;
use tracing::info;

use crate::{
    gateway::gateway,
    routers::{auth_router, user_router},
    state::init_state,
};

mod app;
mod config;
mod core;
mod domain;
mod error;
mod gateway;
mod infra;
mod routers;
mod state;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let app_config = config::config().await;

    let app_state = init_state();

    let app = Router::new()
        .nest("/users", user_router())
        .nest("/auth", auth_router(app_state.clone()))
        .layer(gateway(app_state.clone()));

    let address = format!("0.0.0.0:{}", app_config.port);
    let socket_addr: SocketAddr = address.parse().expect("Unable to parse socket address");

    let listener = tokio::net::TcpListener::bind(socket_addr)
        .await
        .expect("Failed to bind");

    info!("Starting server on {}", address);

    axum::serve(listener, app)
        .await
        .expect("Server failed to start");
}
