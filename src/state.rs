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
        auth::{
            hash_handler::{BcryptHandler, HashHandler},
            jwks_service::FileJwksService,
            jwt_handler::{JwtHandler, JwtService},
            totp_handler::{TotpHandler, TotpService},
        },
        data::{
            attachment_repository::ScyllaAttachmentRepository,
            chat_repotisory::ScyllaChatRepository, message_repository::ScyllaMessageRepository,
            user_repository::ScyllaUserRepository,
            user_session_repository::ScyllaUserSessionRepository,
        },
        id_generator::{IdGenerator, SnowflakeIdGenerator},
        smtp_client::{SmtpClient, SmtpService},
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
    pub hash_handler: Arc<dyn HashHandler>,
    pub jwks_service: Arc<FileJwksService>,
    pub jwt_handler: Arc<dyn JwtHandler>,
    pub smtp_client: Arc<dyn SmtpClient>,
    pub totp_handler: Arc<dyn TotpHandler>,
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

    session.use_keyspace("zinq", true).await;

    let jwks_service = FileJwksService::load_from_directory(&app_config.auth.keys_directory)
        .expect("Failed to init JwksService");

    AppState {
        event_bus: Arc::new(EventBus::new()),
        id_gen: Arc::new(SnowflakeIdGenerator::new()),
        user_repository: Arc::new(ScyllaUserRepository::new(session.clone())),
        user_session_repository: Arc::new(ScyllaUserSessionRepository::new(session.clone())),
        chat_repotisory: Arc::new(ScyllaChatRepository::new(session.clone())),
        message_repository: Arc::new(ScyllaMessageRepository::new(session.clone())),
        attachment_repository: Arc::new(ScyllaAttachmentRepository::new(session.clone())),
        hash_handler: Arc::new(BcryptHandler::new()),
        jwks_service: Arc::new(jwks_service),
        jwt_handler: Arc::new(JwtService::new(
            jwks_service,
            app_config.auth.access_token_expiration_seconds as i64,
        )),
        smtp_client: Arc::new(SmtpService::new(
            app_config.smtp.from.clone(),
            app_config.smtp.host.clone(),
            app_config.smtp.port,
            app_config.smtp.username.clone(),
            app_config.smtp.password.clone(),
        )),
        totp_handler: Arc::new(TotpService::new()),
    }
}
