use std::net::SocketAddr;

use axum::Router;
use tracing::info;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    application::{
        events::start_event_listener,
        meta_messages::start_meta_message_worker,
    },
    gateway::gateway,
    infra::event_bus::NatsEventBus,
    routers::{auth_router, chat_router, emoji_router, user_router, well_known_router},
    state::init_state,
};

#[cfg(test)]
mod tests;

mod application;
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
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug"));

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(env_filter)
        .init();

    info!("Initializing server");

    let app_config = config::config().await;

    let app_state = init_state().await;

    let nats_client = async_nats::connect(&app_config.nats_url)
        .await
        .expect("Failed to connect to NATS");

    let jetstream = async_nats::jetstream::new(nats_client);

    let nats_event_bus = NatsEventBus::new(jetstream.clone());
    nats_event_bus
        .initialize_stream()
        .await
        .expect("Failed to initialize NATS stream");

    let (socket_layer, io) = gateway(app_state.clone());

    start_event_listener(
        jetstream,
        app_state.event_log_repository.clone(),
        io,
    )
    .await
    .expect("Failed to start event listener");

    let app = Router::new()
        .nest("/.well-known", well_known_router(app_state.clone()))
        .nest("/users", user_router(app_state.clone()))
        .nest("/auth", auth_router(app_state.clone()))
        .nest("/emoji-packs", emoji_router(app_state.clone()))
        .nest("/chats", chat_router(app_state.clone()))
        .layer(socket_layer);

    start_meta_message_worker(app_state);

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
