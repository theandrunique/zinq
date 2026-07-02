use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::domain::{
    attachments::Attachment,
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
    ChatMemberAdd {
        member: ChatMember,
        chat_id: i64,
        initiator_id: i64,
    },
    ChatMemberRemove {
        member: ChatMember,
        chat_id: i64,
        initiator_id: i64,
    },
    MessageCreate {
        message: Message,
        attachments: Vec<Attachment>,
        initiator_id: i64,
    },
    MessageUpdate {
        message: Message,
        attachments: Vec<Attachment>,
        initiator_id: i64,
    },
    MessageAck {
        user_id: i64,
        chat_id: i64,
        last_read_message_id: i64,
    },
}

#[async_trait]
pub trait DomainEventHandler: Send + Sync {
    async fn handle(&self, event: &DomainEvent) -> Result<(), anyhow::Error>;
}

pub struct Mediator {
    handlers: RwLock<Vec<Arc<dyn DomainEventHandler>>>,
}

impl Mediator {
    pub fn new() -> Self {
        Self {
            handlers: RwLock::new(Vec::new()),
        }
    }

    pub async fn register<H: DomainEventHandler + 'static>(&self, handler: H) {
        self.handlers.write().await.push(Arc::new(handler));
    }

    pub async fn publish(&self, event: &DomainEvent) -> Result<(), anyhow::Error> {
        let handlers = self.handlers.read().await;

        for handler in handlers.iter() {
            handler.handle(event).await?;
        }

        Ok(())
    }
}
