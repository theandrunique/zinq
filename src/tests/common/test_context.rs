use std::{fs, sync::Arc};

use scylla::client::{session::Session, session_builder::SessionBuilder};

use crate::{
    application::{
        RequestHandler,
        auth::{RegisterComandHandler, RegisterCommand},
        chats::{
            CreateChatCommand, CreateChatCommandHandler, GetDMChatCommand, GetDMChatCommandHandler,
        },
        messages::{
            AddOrEditMessageCommand, AddOrEditMessageCommandHandler, AddOrEditMessageCommandResult,
        },
        services::{AttachmentService, AvatarService, ChannelImageService},
    },
    config::S3Config,
    domain::{
        auth::User,
        chats::{Chat, ChatPermissions, data::ChatLoader},
        event_log::data::EventLogRepository,
        events::EventBus,
        messages::{CreateMessageRequest, Message},
    },
    infra::{
        auth::{
            hash_handler::BcryptHandler, jwks_service::FileJwksService, jwt_handler::JwtService,
            totp_handler::TotpService,
        },
        data::{
            attachment_repository::ScyllaAttachmentRepository, chat_loader::ScyllaChatLoader,
            chat_member_repository::ScyllaChatMemberRepository,
            chat_repotisory::ScyllaChatRepository, event_log_repository::ScyllaEventLogRepository,
            message_repository::ScyllaMessageRepository, user_repository::ScyllaUserRepository,
            user_session_repository::ScyllaUserSessionRepository,
        },
        id_generator::SnowflakeIdGenerator,
        s3::AwsS3Service,
        smtp_client::SmtpService,
    },
    state::AppState,
    tests::common::test_config::{
        S3_ACCESS_KEY_ENV, S3_BUCKET_ENV, S3_ENDPOINT_ENV, S3_REGION_ENV, S3_SECRET_KEY_ENV,
        SCYLLA_HOST_ENV, SCYLLA_PORT_ENV,
    },
};

pub struct TestContext {
    pub app_state: AppState,
}

impl TestContext {
    pub async fn new(keyspace: &str) -> Self {
        let host = std::env::var(SCYLLA_HOST_ENV).expect("TEST_SCYLLA_HOST not set");
        let port: u16 = std::env::var(SCYLLA_PORT_ENV)
            .expect("TEST_SCYLLA_PORT not set")
            .parse()
            .expect("Invalid port");

        let hostname = format!("{host}:{port}");

        let session = Arc::new(
            SessionBuilder::new()
                .known_node(hostname)
                .build()
                .await
                .unwrap(),
        );

        Self::setup_database(session.clone(), keyspace).await;

        session.use_keyspace(keyspace, true).await.unwrap();

        let jwks_service = FileJwksService::load_from_directory("keys").unwrap();
        let jwks_service_clone = FileJwksService::load_from_directory("keys").unwrap();

        let s3_endpoint = std::env::var(S3_ENDPOINT_ENV).expect("TEST_S3_ENDPOINT not set");
        let s3_access_key = std::env::var(S3_ACCESS_KEY_ENV).expect("TEST_S3_ACCESS_KEY not set");
        let s3_secret_key = std::env::var(S3_SECRET_KEY_ENV).expect("TEST_S3_SECRET_KEY not set");
        let s3_bucket = std::env::var(S3_BUCKET_ENV).expect("TEST_S3_BUCKET not set");
        let s3_region = std::env::var(S3_REGION_ENV).unwrap_or("us-east-1".to_string());

        let s3_config = S3Config {
            access_key: s3_access_key,
            secret_key: s3_secret_key,
            service_url: s3_endpoint,
            force_path_style: true,
            bucket_name: s3_bucket,
            region: s3_region,
        };

        let s3_service: Arc<dyn crate::infra::s3::S3Service> =
            Arc::new(AwsS3Service::new(&s3_config).await);

        let _ = s3_service.create_bucket(&s3_config.bucket_name).await;

        let id_gen = Arc::new(SnowflakeIdGenerator::new());

        let attachment_service = Arc::new(AttachmentService::new(
            s3_service.clone(),
            id_gen.clone(),
            &s3_config,
        ));

        let avatar_service = Arc::new(AvatarService::new(s3_service.clone(), &s3_config));

        let channel_image_service =
            Arc::new(ChannelImageService::new(s3_service.clone(), &s3_config));

        let state = AppState {
            event_bus: Arc::new(EventBus::new()),
            event_log_repository: Arc::new(ScyllaEventLogRepository::new(session.clone())),
            id_gen,
            user_repository: Arc::new(ScyllaUserRepository::new(session.clone())),
            user_session_repository: Arc::new(ScyllaUserSessionRepository::new(session.clone())),
            chat_loader: Arc::new(ScyllaChatLoader::new(session.clone())),
            chat_member_repository: Arc::new(ScyllaChatMemberRepository::new(session.clone())),
            chat_repository: Arc::new(ScyllaChatRepository::new(session.clone())),
            message_repository: Arc::new(ScyllaMessageRepository::new(session.clone())),
            attachment_repository: Arc::new(ScyllaAttachmentRepository::new(session.clone())),
            hash_handler: Arc::new(BcryptHandler::new()),
            jwks_service: Arc::new(jwks_service),
            jwt_handler: Arc::new(JwtService::new(jwks_service_clone, 3600)),
            smtp_client: Arc::new(SmtpService::new(
                "test@test.com".to_string(),
                "smtp.test.com".to_string(),
                587,
                "test".to_string(),
                "test".to_string(),
            )),
            totp_handler: Arc::new(TotpService::new()),
            s3_service,
            attachment_service,
            avatar_service,
            channel_image_service,
        };

        Self { app_state: state }
    }

    async fn setup_database(session: Arc<Session>, keyspace: &str) {
        session.query_unpaged(
            format!("CREATE KEYSPACE IF NOT EXISTS {} WITH replication = {{'class': 'SimpleStrategy', 'replication_factor': 1}}", keyspace),
            &[]
        ).await.expect("Failed to create keyspace");

        session.use_keyspace(keyspace, true).await.unwrap();

        let migration_cql =
            fs::read_to_string("migrations/1.cql").expect("Failed to read migration file");

        let queries: Vec<&str> = migration_cql.split(';').collect();

        for query in queries {
            let trimmed = query.trim();
            if !trimmed.is_empty() {
                session
                    .query_unpaged(trimmed, &[])
                    .await
                    .expect(&format!("Failed to execute migration: {}", trimmed));
            }
        }
    }

    pub async fn create_test_user(&self, username: &str, email: &str) -> User {
        let cmd = RegisterCommand {
            username: username.to_string(),
            password: "Test123!".to_string(),
            global_name: username.to_string(),
            email: email.to_string(),
        };

        let handler = RegisterComandHandler::new(&self.app_state);
        handler
            .handle(cmd)
            .await
            .expect(&format!("Failed to create test user {}", username))
    }

    pub async fn create_group_chat(
        &self,
        owner_id: i64,
        name: &str,
        members: Vec<i64>,
        permissions: Option<ChatPermissions>,
    ) -> Chat {
        let cmd = CreateChatCommand {
            current_user_id: owner_id,
            name: name.to_string(),
            members,
            permissions,
        };

        let handler = CreateChatCommandHandler::new(&self.app_state);
        handler
            .handle(cmd)
            .await
            .expect(&format!("Failed to create group chat {}", name))
    }

    pub async fn get_or_create_dm_chat(&self, user1_id: i64, user2_id: i64) -> Chat {
        let cmd = GetDMChatCommand {
            current_user_id: user1_id,
            user_id: user2_id,
        };

        let handler = GetDMChatCommandHandler::new(&self.app_state);
        handler.handle(cmd).await.expect(&format!(
            "Failed to get DM chat between {} and {}",
            user1_id, user2_id
        ))
    }

    pub async fn create_message(
        &self,
        chat_id: i64,
        author_id: i64,
        content: &str,
    ) -> AddOrEditMessageCommandResult {
        let cmd = AddOrEditMessageCommand {
            current_user_id: author_id,
            message_id: None,
            referenced_message_id: None,
            chat_id: chat_id,
            content: content.to_string(),
            attachments: vec![],
        };

        let msg_handler = AddOrEditMessageCommandHandler::new(&self.app_state);

        msg_handler
            .handle(cmd)
            .await
            .expect("Failed to create message")
    }
}
