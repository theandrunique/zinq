use std::process::{Command, ExitCode};

use testcontainers::{ImageExt, runners::AsyncRunner};
use testcontainers_modules::minio::MinIO;
use testcontainers_modules::nats::{Nats, NatsServerCmd};
use testcontainers_modules::scylladb::ScyllaDB;

#[tokio::main]
async fn main() -> ExitCode {
    let scylla = match ScyllaDB::default().with_tag("6.2.3").start().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to start ScyllaDB: {}", e);
            return ExitCode::FAILURE;
        }
    };

    let minio = match MinIO::default()
        .with_env_var("MINIO_DEFAULT_BUCKETS", "test-bucket")
        .start()
        .await
    {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Failed to start MinIO: {}", e);
            return ExitCode::FAILURE;
        }
    };

    let scylla_host = scylla.get_host().await.unwrap().to_string();
    let scylla_port = scylla.get_host_port_ipv4(9042).await.unwrap().to_string();

    let minio_host = minio.get_host().await.unwrap().to_string();
    let minio_port = minio.get_host_port_ipv4(9000).await.unwrap();

    let minio_url = format!("http://{}:{}", minio_host, minio_port);

    let nats = match Nats::default()
        .with_cmd(&NatsServerCmd::default().with_jetstream())
        .start()
        .await
    {
        Ok(n) => n,
        Err(e) => {
            eprintln!("Failed to start NATS: {}", e);
            return ExitCode::FAILURE;
        }
    };

    let nats_host = nats.get_host().await.unwrap().to_string();
    let nats_port = nats.get_host_port_ipv4(4222).await.unwrap();

    let nats_url = format!("nats://{}:{}", nats_host, nats_port);

    let status = Command::new("cargo")
        .arg("test")
        .env("TEST_SCYLLA_HOST", scylla_host)
        .env("TEST_SCYLLA_PORT", scylla_port.to_string())
        .env("TEST_S3_ENDPOINT", minio_url)
        .env("TEST_S3_ACCESS_KEY", "minioadmin")
        .env("TEST_S3_SECRET_KEY", "minioadmin")
        .env("TEST_S3_BUCKET", "test-bucket")
        .env("TEST_S3_REGION", "us-east-1")
        .env("TEST_NATS_URL", nats_url)
        .spawn()
        .expect("Failed to start cargo test")
        .wait()
        .expect("Failed to wait on cargo test");

    if status.success() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
