use std::{cmp, sync::Arc};

use chrono::{DateTime, Duration, TimeZone, Utc};

use crate::{
    application::RequestHandler,
    domain::{
        chats::data::{ChatLoadOptions, ChatLoader, ChatRepository},
        events::{DomainEvent, Mediator},
        message_acks::{MessageAck, data::MessageAckRepository},
        messages::data::MessageRepository,
    },
    error::Error,
    state::AppState,
};

pub struct MessageAckCommand {
    pub current_user_id: i64,
    pub chat_id: i64,
    pub last_read_message_id: i64,
    pub acks: Vec<MessageAckInput>,
}

pub struct MessageAckInput {
    pub message_id: i64,
    pub acked_at: DateTime<Utc>,
}

pub struct MessageAckCommandHandler {
    chat_loader: Arc<dyn ChatLoader>,
    chat_repository: Arc<dyn ChatRepository>,
    message_ack_repository: Arc<dyn MessageAckRepository>,
    message_repository: Arc<dyn MessageRepository>,
    mediator: Arc<Mediator>,
}

impl MessageAckCommandHandler {
    pub fn new(app_state: &AppState) -> Self {
        Self {
            chat_loader: Arc::clone(&app_state.chat_loader),
            chat_repository: Arc::clone(&app_state.chat_repository),
            message_ack_repository: Arc::clone(&app_state.message_ack_repository),
            message_repository: Arc::clone(&app_state.message_repository),
            mediator: Arc::clone(&app_state.mediator),
        }
    }
}

impl RequestHandler for MessageAckCommandHandler {
    type Request = MessageAckCommand;
    type Output = ();
    type Error = Error;

    async fn handle(&self, request: Self::Request) -> Result<Self::Output, Self::Error> {
        let chat = self
            .chat_loader
            .load(
                ChatLoadOptions::default()
                    .with_chat_id(request.chat_id)
                    .with_member(request.current_user_id),
            )
            .await
            .map_err(Error::InternalServerError)?
            .ok_or(Error::ChatNotFound(request.chat_id))?;

        let member = chat
            .get_member(request.current_user_id)
            .ok_or(Error::UserNotMember {
                user_id: request.current_user_id,
                chat_id: request.chat_id,
            })?;

        let current_last_read = member.last_read_message_id.unwrap_or(0);
        if request.last_read_message_id <= current_last_read {
            return Ok(());
        }

        self.chat_repository
            .update_last_read_message_id(
                request.current_user_id,
                request.chat_id,
                request.last_read_message_id,
            )
            .await?;

        let mut acks: Vec<MessageAck> = Vec::new();
        let now = Utc::now();
        let epoch = Utc.with_ymd_and_hms(2005, 5, 20, 0, 0, 0).unwrap();

        if request.acks.is_empty() {
            let min_id = ((Utc::now() - Duration::days(7)) - epoch).num_milliseconds() << 22;
            let range_start = cmp::max(current_last_read + 1, min_id);

            if range_start <= request.last_read_message_id {
                let message_ids = self
                    .message_repository
                    .get_message_ids_in_range(
                        request.chat_id,
                        range_start,
                        request.last_read_message_id,
                    )
                    .await?;

                for message_id in message_ids {
                    acks.push(MessageAck {
                        chat_id: request.chat_id,
                        message_id,
                        user_id: request.current_user_id,
                        created_at: now,
                    });
                }
            }
        } else {
            let cutoff_millis = (Utc::now() - Duration::days(7) - epoch).num_milliseconds();

            for ack_input in &request.acks {
                if ack_input.message_id >> 22 >= cutoff_millis {
                    acks.push(MessageAck {
                        chat_id: request.chat_id,
                        message_id: ack_input.message_id,
                        user_id: request.current_user_id,
                        created_at: ack_input.acked_at,
                    });
                }
            }
        }

        if !acks.is_empty() {
            self.message_ack_repository.bulk_upsert(&acks).await?;
        }

        self.mediator
            .publish(&DomainEvent::MessageAck {
                user_id: request.current_user_id,
                chat_id: request.chat_id,
                last_read_message_id: request.last_read_message_id,
            })
            .await?;

        Ok(())
    }
}
