use std::sync::Arc;

use anyhow::Context;
use async_trait::async_trait;

use crate::{
    domain::{
        events::{DomainEvent, DomainEventHandler, Mediator},
        messages::{CreateMetaMessageRequest, Message, MessageType, data::MessageRepository},
    },
    infra::IdGenerator,
    state::AppState,
};

pub struct ChatMemberAddedMetaMessage {
    mediator: Arc<Mediator>,
    message_repository: Arc<dyn MessageRepository>,
    id_gen: Arc<dyn IdGenerator>,
}

impl ChatMemberAddedMetaMessage {
    pub fn new(app_state: &AppState) -> Self {
        Self {
            mediator: Arc::clone(&app_state.mediator),
            message_repository: Arc::clone(&app_state.message_repository),
            id_gen: Arc::clone(&app_state.id_gen),
        }
    }
}

#[async_trait]
impl DomainEventHandler for ChatMemberAddedMetaMessage {
    async fn handle(&self, event: &DomainEvent) -> Result<(), anyhow::Error> {
        let DomainEvent::ChatMemberAdd {
            chat_id,
            member,
            initiator_id,
        } = event
        else {
            return Ok(());
        };

        let meta_message = Message::new_meta(CreateMetaMessageRequest {
            id: self.id_gen.gen_id().await,
            chat_id: *chat_id,
            author_id: *initiator_id,
            message_type: MessageType::MemberAdd {
                user_id: member.user_id,
            },
        });

        self.message_repository.upsert(&meta_message).await?;

        self.mediator
            .publish(&DomainEvent::MessageCreate {
                message: meta_message,
                attachments: vec![],
                initiator_id: *initiator_id,
            })
            .await?;

        Ok(())
    }
}
