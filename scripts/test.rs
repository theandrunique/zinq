use std::process::{Command, ExitCode};

use anyhow::Result;
use testcontainers::{ImageExt, runners::AsyncRunner};
use testcontainers_modules::scylladb::ScyllaDB;

#[tokio::main]
async fn main() -> ExitCode {
    let scylla = ScyllaDB::default().with_tag("6.2.3").start().await.unwrap();

    let status = Command::new("cargo")
        .arg("test")
        .env(
            "TEST_SCYLLA_HOST",
            scylla.get_host().await.unwrap().to_string(),
        )
        .env(
            "TEST_SCYLLA_PORT",
            scylla.get_host_port_ipv4(9042).await.unwrap().to_string(),
        )
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
