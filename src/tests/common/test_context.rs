use std::{fs, sync::Arc};

use scylla::client::{session::Session, session_builder::SessionBuilder};

use crate::{
    domain::events::EventBus,
    infra::{
        auth::{
            hash_handler::BcryptHandler, jwks_service::FileJwksService,
            jwt_handler::JwtService, totp_handler::TotpService,
        },
        data::{
            attachment_repository::ScyllaAttachmentRepository,
            chat_repotisory::ScyllaChatRepository, message_repository::ScyllaMessageRepository,
            user_repository::ScyllaUserRepository,
            user_session_repository::ScyllaUserSessionRepository,
        },
        id_generator::SnowflakeIdGenerator,
        smtp_client::SmtpService,
    },
    state::AppState, tests::common::test_config::{SCYLLA_HOST_ENV, SCYLLA_PORT_ENV},
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

        let state = AppState {
            event_bus: Arc::new(EventBus::new()),
            id_gen: Arc::new(SnowflakeIdGenerator::new()),
            user_repository: Arc::new(ScyllaUserRepository::new(session.clone())),
            user_session_repository: Arc::new(ScyllaUserSessionRepository::new(
                session.clone(),
            )),
            chat_repotisory: Arc::new(ScyllaChatRepository::new(session.clone())),
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

    pub async fn create_test_user(
        &self,
        username: &str,
        email: &str,
    ) -> crate::domain::auth::User {
        use crate::application::RequestHandler;
        use crate::application::auth::{RegisterComandHandler, RegisterCommand};

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
}
