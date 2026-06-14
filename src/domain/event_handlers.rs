use std::{any::Any, sync::Arc};

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::domain::events::DomainEvent;

#[async_trait]
pub trait DomainEventHandler: Send + Sync {
    async fn handle(&self, event: &DomainEvent) -> Result<(), anyhow::Error>;
}

pub struct EventBus {
    handlers: Vec<Arc<dyn DomainEventHandler>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self { handlers: Vec::new() }
    }

    pub fn register<H: DomainEventHandler + 'static>(&mut self, handler: H) {
        self.handlers.push(Arc::new(handler));
    }

    pub async fn publish(&self, event: &DomainEvent) -> Result<(), anyhow::Error> {
        for handler in self.handlers.iter() {
            handler.handle(event).await?;
        }
        Ok(())
    }
}
