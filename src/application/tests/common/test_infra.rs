use once_cell::sync::Lazy;
use std::sync::Arc;
use testcontainers::{ContainerAsync, ImageExt, runners::AsyncRunner};
use testcontainers_modules::scylladb::ScyllaDB;
use tokio::sync::Mutex;

pub struct TestInfra {
    pub scylla: ContainerAsync<ScyllaDB>,
}

impl TestInfra {
    async fn new() -> Self {
        let scylla = ScyllaDB::default().with_tag("6.2.3").start().await.unwrap();

        Self { scylla }
    }
}

static INFRA: Lazy<Mutex<Option<Arc<TestInfra>>>> = Lazy::new(|| Mutex::new(None));

pub async fn get_infra() -> Arc<TestInfra> {
    let mut guard = INFRA.lock().await;

    if let Some(infra) = &*guard {
        return infra.clone();
    }

    let infra = Arc::new(TestInfra::new().await);
    *guard = Some(infra.clone());
    infra
}

pub async fn shutdown_infra() {
    let mut guard = INFRA.lock().await;
    *guard = None;
}
