use std::sync::Arc;

use scylla::client::session_builder::SessionBuilder;

use crate::{
    application::{
        meta_messages::ChatCreateMetaMessage,
        services::{AttachmentService, AvatarService, ChannelImageService},
    },
    config,
    domain::{
        attachments::data::AttachmentRepository,
        auth::data::{
            user_repository::UserRepository, user_session_repository::UserSessionRepository,
        },
        chats::data::{ChatLoader, ChatMemberRepository, ChatRepository},
        event_log::data::EventLogRepository,
        events::Mediator,
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
            attachment_repository::ScyllaAttachmentRepository, chat_loader::ScyllaChatLoader,
            chat_member_repository::ScyllaChatMemberRepository,
            chat_repotisory::ScyllaChatRepository, event_log_repository::ScyllaEventLogRepository,
            message_repository::ScyllaMessageRepository, user_repository::ScyllaUserRepository,
            user_session_repository::ScyllaUserSessionRepository,
        },
        event_bus::{EventBus, NatsEventBus},
        id_generator::{IdGenerator, SnowflakeIdGenerator},
        s3::{AwsS3Service, S3Service},
        smtp_client::{SmtpClient, SmtpService},
    },
};

#[derive(Clone)]
pub struct AppState {
    pub event_bus: Arc<dyn EventBus>,
    pub event_log_repository: Arc<dyn EventLogRepository>,
    pub id_gen: Arc<dyn IdGenerator>,
    pub user_repository: Arc<dyn UserRepository>,
    pub user_session_repository: Arc<dyn UserSessionRepository>,
    pub chat_loader: Arc<dyn ChatLoader>,
    pub chat_member_repository: Arc<dyn ChatMemberRepository>,
    pub chat_repository: Arc<dyn ChatRepository>,
    pub message_repository: Arc<dyn MessageRepository>,
    pub attachment_repository: Arc<dyn AttachmentRepository>,
    pub hash_handler: Arc<dyn HashHandler>,
    pub jwks_service: Arc<FileJwksService>,
    pub jwt_handler: Arc<dyn JwtHandler>,
    pub smtp_client: Arc<dyn SmtpClient>,
    pub totp_handler: Arc<dyn TotpHandler>,
    pub s3_service: Arc<dyn S3Service>,
    pub attachment_service: Arc<AttachmentService>,
    pub avatar_service: Arc<AvatarService>,
    pub channel_image_service: Arc<ChannelImageService>,
    pub mediator: Arc<Mediator>,
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

    session
        .use_keyspace("zinq", true)
        .await
        .expect("Failed to use keyspace");

    let jwks_service = FileJwksService::load_from_directory(&app_config.auth.keys_directory)
        .expect("Failed to init JwksService");

    let s3_service: Arc<dyn S3Service> = Arc::new(AwsS3Service::new(&app_config.s3).await);

    let id_gen: Arc<dyn IdGenerator> = Arc::new(SnowflakeIdGenerator::new());

    let attachment_service = Arc::new(AttachmentService::new(
        s3_service.clone(),
        id_gen.clone(),
        &app_config.s3,
    ));

    let avatar_service = Arc::new(AvatarService::new(s3_service.clone(), &app_config.s3));

    let channel_image_service =
        Arc::new(ChannelImageService::new(s3_service.clone(), &app_config.s3));

    let client = async_nats::connect(&app_config.nats_url).await.unwrap();
    let jetstream = async_nats::jetstream::new(client);

    AppState {
        event_bus: Arc::new(NatsEventBus::new(jetstream)),
        event_log_repository: Arc::new(ScyllaEventLogRepository::new(session.clone())),
        id_gen: id_gen.clone(),
        user_repository: Arc::new(ScyllaUserRepository::new(session.clone())),
        user_session_repository: Arc::new(ScyllaUserSessionRepository::new(session.clone())),
        chat_loader: Arc::new(ScyllaChatLoader::new(session.clone())),
        chat_member_repository: Arc::new(ScyllaChatMemberRepository::new(session.clone())),
        chat_repository: Arc::new(ScyllaChatRepository::new(session.clone())),
        message_repository: Arc::new(ScyllaMessageRepository::new(session.clone())),
        attachment_repository: Arc::new(ScyllaAttachmentRepository::new(session.clone())),
        hash_handler: Arc::new(BcryptHandler::new()),
        jwks_service: Arc::new(jwks_service.clone()),
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
        s3_service,
        attachment_service,
        avatar_service,
        channel_image_service,
        mediator: Arc::new(Mediator::new()),
    }
}
