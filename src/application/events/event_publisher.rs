use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;

use crate::{
    domain::{
        chats::data::ChatRepository,
        event_log::{Event, EventLogType},
        events::{DomainEvent, DomainEventHandler},
    },
    infra::{IdGenerator, event_bus::EventBus},
    state::AppState,
};

pub struct EventPublisher {
    event_bus: Arc<dyn EventBus>,
    id_gen: Arc<dyn IdGenerator>,
    chat_repository: Arc<dyn ChatRepository>,
}

impl EventPublisher {
    pub fn new(app_state: &AppState) -> Self {
        Self {
            event_bus: Arc::clone(&app_state.event_bus),
            id_gen: Arc::clone(&app_state.id_gen),
            chat_repository: Arc::clone(&app_state.chat_repository),
        }
    }
}

#[async_trait]
impl DomainEventHandler for EventPublisher {
    async fn handle(&self, event: &DomainEvent) -> Result<(), anyhow::Error> {
        let (event_type, chat_id) = match event {
            DomainEvent::ChatCreate { chat } => {
                (EventLogType::ChatCreate { chat: chat.clone() }, chat.id)
            }
            DomainEvent::ChatMemberAdd { chat_id, member, .. } => (
                EventLogType::ChatMemberAdd {
                    chat_id: *chat_id,
                    member: member.clone(),
                },
                *chat_id,
            ),
            DomainEvent::ChatMemberRemove { chat_id, member, .. } => (
                EventLogType::ChatMemberRemove {
                    chat_id: *chat_id,
                    member: member.clone(),
                },
                *chat_id,
            ),
            DomainEvent::MessageCreate {
                message,
                attachments,
                ..
            } => (
                EventLogType::MessageCreate {
                    message: message.clone(),
                    attachments: attachments.clone(),
                },
                message.chat_id,
            ),
            DomainEvent::MessageUpdate {
                message,
                attachments,
                ..
            } => (
                EventLogType::MessageUpdate {
                    message: message.clone(),
                    attachments: attachments.clone(),
                },
                message.chat_id,
            ),
            _ => return Ok(()),
        };

        let recipients = self
            .chat_repository
            .get_member_ids(chat_id)
            .await?
            .into_iter()
            .filter(|(_, is_leave)| !is_leave)
            .map(|(user_id, _)| user_id)
            .collect::<Vec<_>>();

        let log_event = Event {
            event_id: self.id_gen.gen_id().await,
            event_type,
            created_at: Utc::now(),
            recipients,
        };

        self.event_bus.publish(log_event).await?;

        Ok(())
    }
}
