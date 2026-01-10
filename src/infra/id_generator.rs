use std::time::UNIX_EPOCH;

use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use tokio::sync::Mutex;

#[async_trait]
pub trait IdGenerator: Send + Sync {
    async fn gen_id(&self) -> i64;
}

pub struct SnowflakeIdGenerator {
    snowflake: Mutex<snowflake::SnowflakeIdGenerator>,
}

impl SnowflakeIdGenerator {
    pub fn new() -> Self {
        let dt = Utc
            .with_ymd_and_hms(2005, 5, 20, 0, 0, 0)
            .single()
            .expect("Invalid epoch");

        let epoch = UNIX_EPOCH + std::time::Duration::from_millis(dt.timestamp_millis() as u64);

        let machine_id = 31;
        let node_id = 24;

        Self {
            snowflake: Mutex::new(snowflake::SnowflakeIdGenerator::with_epoch(
                machine_id, node_id, epoch,
            )),
        }
    }
}

#[async_trait]
impl IdGenerator for SnowflakeIdGenerator {
    async fn gen_id(&self) -> i64 {
        self.snowflake.lock().await.real_time_generate()
    }
}
