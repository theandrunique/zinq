use std::sync::Arc;

use anyhow::Context;
use async_trait::async_trait;

use crate::{
    domain::{
        chats::ChatType,
        events::{DomainEvent, DomainEventHandler, Mediator},
        messages::{CreateMetaMessageRequest, Message, MessageType, data::MessageRepository},
    },
    infra::IdGenerator,
    state::AppState,
};

pub struct ChatCreateMetaMessage {
    mediator: Arc<Mediator>,
    message_repository: Arc<dyn MessageRepository>,
    id_gen: Arc<dyn IdGenerator>,
}

impl ChatCreateMetaMessage {
    pub fn new(app_state: &AppState) -> Self {
        Self {
            mediator: Arc::clone(&app_state.mediator),
            message_repository: Arc::clone(&app_state.message_repository),
            id_gen: Arc::clone(&app_state.id_gen),
        }
    }
}

#[async_trait]
impl DomainEventHandler for ChatCreateMetaMessage {
    async fn handle(&self, event: &DomainEvent) -> Result<(), anyhow::Error> {
        let DomainEvent::ChatCreate { chat } = event else {
            return Ok(());
        };

        if chat.chat_type != ChatType::GroupDm {
            return Ok(());
        }

        let name = chat
            .name
            .as_ref()
            .with_context(|| format!("Chat name was None for {}", chat.id))?;

        let owner_id = chat
            .owner_id
            .with_context(|| format!("Chat owner_id was None for {}", chat.id))?;

        let member = chat
            .get_member(owner_id)
            .with_context(|| format!("Chat member not found for owner_id {}", owner_id))?;

        let meta_message = Message::new_meta(CreateMetaMessageRequest {
            id: self.id_gen.gen_id().await,
            chat_id: chat.id,
            author_id: owner_id,
            message_type: MessageType::ChatCreate {
                chat_name: name.clone(),
            },
        });

        self.message_repository.upsert(meta_message.clone()).await?;
        self.mediator
            .publish(&DomainEvent::MessageCreated {
                chat: chat.clone(),
                message: meta_message,
                member,
                attachments: vec![],
            })
            .await?;

        Ok(())
    }
}
