use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::broadcast;

use crate::domain::{
    auth::User,
    chats::{Chat, ChatMember},
    messages::Message,
};

#[derive(Clone, Debug)]
pub enum DomainEvent {
    UserCreate {
        user: User,
    },
    ChatCreate {
        chat: Chat,
    },
    ChatMemberAdded {
        chat: Chat,
        member: ChatMember,
        initiator_id: i64,
    },
    ChatMemberRemoved {
        chat: Chat,
        member: ChatMember,
        initiator_id: i64,
    },
    MessageCreated {
        chat: Chat,
        message: Message,
        member: ChatMember,
    },
    MessageUpdated {
        chat: Chat,
        message: Message,
        member: ChatMember,
    },
}

#[async_trait]
pub trait DomainEventHandler: Send + Sync {
    async fn handle(&self, event: &DomainEvent) -> Result<(), anyhow::Error>;
}

pub struct Mediator {
    handlers: Vec<Arc<dyn DomainEventHandler>>,
}

impl Mediator {
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
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
