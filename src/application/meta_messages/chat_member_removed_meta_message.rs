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

pub struct ChatMemberRemovedMetaMessage {
    mediator: Arc<Mediator>,
    message_repository: Arc<dyn MessageRepository>,
    id_gen: Arc<dyn IdGenerator>,
}

impl ChatMemberRemovedMetaMessage {
    pub fn new(app_state: &AppState) -> Self {
        Self {
            mediator: Arc::clone(&app_state.mediator),
            message_repository: Arc::clone(&app_state.message_repository),
            id_gen: Arc::clone(&app_state.id_gen),
        }
    }
}

#[async_trait]
impl DomainEventHandler for ChatMemberRemovedMetaMessage {
    async fn handle(&self, event: &DomainEvent) -> Result<(), anyhow::Error> {
        let DomainEvent::ChatMemberRemove {
            chat_id,
            member,
            initiator_id,
        } = event
        else {
            return Ok(());
        };

        let initiator_id_val = *initiator_id;

        let message_type = if member.user_id == initiator_id_val {
            MessageType::MemberLeave {
                user_id: initiator_id_val,
            }
        } else {
            MessageType::MemberRemove {
                user_id: member.user_id,
            }
        };

        let meta_message = Message::new_meta(CreateMetaMessageRequest {
            id: self.id_gen.gen_id().await,
            chat_id: *chat_id,
            author_id: initiator_id_val,
            message_type,
        });

        self.message_repository.upsert(&meta_message).await?;

        self.mediator
            .publish(&DomainEvent::MessageCreate {
                message: meta_message,
                attachments: vec![],
                initiator_id: initiator_id_val,
            })
            .await?;

        Ok(())
    }
}
