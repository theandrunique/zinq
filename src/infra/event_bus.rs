use async_nats::{
    HeaderMap,
    jetstream::{
        Context,
        stream::{Config, DiscardPolicy},
    },
};
use async_trait::async_trait;

use crate::domain::event_log::Event;

#[async_trait]
pub trait EventBus: Send + Sync {
    async fn publish(&self, event: Event) -> Result<(), anyhow::Error>;
}

pub struct NatsEventBus {
    jetstream: Context,
}

impl NatsEventBus {
    pub fn new(jetstream: Context) -> Self {
        Self { jetstream }
    }

    pub async fn initialize_stream(&self) -> Result<(), anyhow::Error> {
        self.jetstream
            .create_or_update_stream(Config {
                name: "events".to_string(),
                discard: DiscardPolicy::New,
                subjects: vec!["events.>".to_string()],
                ..Default::default()
            })
            .await?;
        Ok(())
    }
}

#[async_trait]
impl EventBus for NatsEventBus {
    async fn publish(&self, event: Event) -> Result<(), anyhow::Error> {
        let payload = serde_json::to_vec(&event)?;

        let event_id = event.event_id;
        let mut headers = HeaderMap::new();
        headers.insert("Nats-Msg-Id", event_id.to_string());

        let ack = self
            .jetstream
            .publish_with_headers("events".to_string(), headers, payload.into())
            .await?
            .await?;

        if ack.duplicate {
            tracing::warn!("Duplicate message published: {}", event_id);
        } else {
            tracing::info!("Message published: {}", event_id);
        }

        Ok(())
    }
}
