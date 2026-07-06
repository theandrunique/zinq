use std::time::{Duration, UNIX_EPOCH};

use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};
use tokio::sync::Mutex;

#[async_trait]
pub trait IdGenerator: Send + Sync {
    async fn gen_id(&self) -> i64;
    fn get_epoch(&self) -> DateTime<Utc>;
    fn extract_timestamp(&self, id: i64) -> DateTime<Utc> {
        self.get_epoch() + chrono::Duration::milliseconds(id >> 22)
    }

    fn min_id_for_days_ago(&self, days: i64) -> i64 {
        let cutoff = Utc::now() - chrono::Duration::days(days);
        ((cutoff - self.get_epoch()).num_milliseconds()) << 22
    }
}

pub struct SnowflakeIdGenerator {
    snowflake: Mutex<snowflake::SnowflakeIdGenerator>,
    epoch: DateTime<Utc>,
}

impl SnowflakeIdGenerator {
    pub fn new() -> Self {
        let dt = Utc
            .with_ymd_and_hms(2005, 5, 20, 0, 0, 0)
            .single()
            .expect("Invalid epoch");

        let epoch = UNIX_EPOCH + Duration::from_millis(dt.timestamp_millis() as u64);

        let machine_id = 31;
        let node_id = 24;

        Self {
            snowflake: Mutex::new(snowflake::SnowflakeIdGenerator::with_epoch(
                machine_id, node_id, epoch,
            )),
            epoch: dt,
        }
    }
}

#[async_trait]
impl IdGenerator for SnowflakeIdGenerator {
    async fn gen_id(&self) -> i64 {
        self.snowflake.lock().await.real_time_generate()
    }

    fn get_epoch(&self) -> DateTime<Utc> {
        self.epoch
    }
}
