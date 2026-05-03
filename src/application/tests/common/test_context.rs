pub mod integration_tests {
    use std::{fs, sync::Arc};

    use scylla::client::{session::Session, session_builder::SessionBuilder};

    use crate::{
        application::tests::common::test_infra::get_infra,
        domain::events::EventBus,
        infra::{
            data::{
                attachment_repository::ScyllaAttachmentRepository,
                chat_repotisory::ScyllaChatRepository, message_repository::ScyllaMessageRepository,
                user_repository::ScyllaUserRepository,
                user_session_repository::ScyllaUserSessionRepository,
            },
            hash_handler::BcryptHandler,
            id_generator::SnowflakeIdGenerator,
            jwt_handler::JwtService,
            smtp_client::SmtpService,
            totp_handler::TotpService,
        },
        state::AppState,
    };

    pub struct TestContext {
        pub app_state: AppState,
    }

    impl TestContext {
        pub async fn new(keyspace: &str) -> Self {
            let infra = get_infra().await;

            let host = infra.scylla.get_host().await.unwrap();
            let port = infra.scylla.get_host_port_ipv4(9042).await.unwrap();
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
                jwt_handler: Arc::new(JwtService::new("test_secret".to_string(), 3600)),
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
    }
}
