use std::sync::Arc;

use scylla::client::session_builder::SessionBuilder;

use crate::{
    config,
    domain::{
        attachments::data::AttachmentRepository,
        auth::data::{
            user_repository::UserRepository, user_session_repository::UserSessionRepository,
        },
        chats::data::ChatRepository,
        events::EventBus,
        messages::data::MessageRepository,
    },
    infra::{
        data::{
            attachment_repository::ScyllaAttachmentRepository,
            chat_repotisory::ScyllaChatRepository, message_repository::ScyllaMessageRepository,
            user_repository::ScyllaUserRepository,
            user_session_repository::ScyllaUserSessionRepository,
        },
        id_generator::{IdGenerator, SnowflakeIdGenerator},
    },
};

#[derive(Clone)]
pub struct AppState {
    pub event_bus: Arc<EventBus>,
    pub id_gen: Arc<dyn IdGenerator>,
    pub user_repository: Arc<dyn UserRepository>,
    pub user_session_repository: Arc<dyn UserSessionRepository>,
    pub chat_repotisory: Arc<dyn ChatRepository>,
    pub message_repository: Arc<dyn MessageRepository>,
    pub attachment_repository: Arc<dyn AttachmentRepository>,
}

pub async fn init_state() -> AppState {
    let app_config = config::config().await;

    let session = Arc::new(
        SessionBuilder::new()
            .known_node(&app_config.scylla_node)
            .build()
            .await
            .expect("Error creating scylla session"),
    );

    AppState {
        event_bus: Arc::new(EventBus::new()),
        id_gen: Arc::new(SnowflakeIdGenerator::new()),
        user_repository: Arc::new(ScyllaUserRepository::new(session.clone())),
        user_session_repository: Arc::new(ScyllaUserSessionRepository::new(session.clone())),
        chat_repotisory: Arc::new(ScyllaChatRepository::new(session.clone())),
        message_repository: Arc::new(ScyllaMessageRepository::new(session.clone())),
        attachment_repository: Arc::new(ScyllaAttachmentRepository::new(session.clone())),
    }
}
