use std::sync::Arc;
use std::time::Duration;

use async_nats::jetstream::{Context, consumer};
use chrono::{DateTime, Utc};
use futures_util::StreamExt;
use serde::Serialize;
use socketioxide::SocketIo;

use crate::{
    domain::event_log::{Event, EventLog, EventLogType, data::EventLogRepository},
    routers::schemas::common::{ChatMemberSchema, ChatSchema, MessageSchema},
};

#[derive(Serialize)]
struct EventLogSchema {
    pub user_id: String,
    pub event_id: String,
    pub event_type: EventLogTypeSchema,
    pub created_at: DateTime<Utc>,
}

impl From<EventLog> for EventLogSchema {
    fn from(event_log: EventLog) -> Self {
        EventLogSchema {
            user_id: event_log.user_id.to_string(),
            event_id: event_log.event_id.to_string(),
            event_type: event_log.event_type.into(),
            created_at: event_log.created_at.clone(),
        }
    }
}

#[derive(Serialize)]
#[serde(tag = "type", content = "payload")]
enum EventLogTypeSchema {
    MessageCreate {
        message: MessageSchema,
    },
    MessageUpdate {
        message: MessageSchema,
    },
    MessageDelete {
        message_id: String,
    },
    ChatCreate {
        chat: ChatSchema,
    },
    ChatMemberAdded {
        chat_id: String,
        member: ChatMemberSchema,
    },
    ChatMemberRemoved {
        chat_id: String,
        member: ChatMemberSchema,
    },
}

impl From<EventLogType> for EventLogTypeSchema {
    fn from(event_type: EventLogType) -> Self {
        match event_type {
            EventLogType::MessageCreate { message, attachments } => EventLogTypeSchema::MessageCreate {
                message: (message, attachments).into(),
            },
            EventLogType::MessageUpdate { message, attachments } => EventLogTypeSchema::MessageUpdate {
                message: (message, attachments).into(),
            },
            EventLogType::MessageDelete { message_id } => {
                EventLogTypeSchema::MessageDelete { message_id: message_id.to_string() }
            }
            EventLogType::ChatCreate { chat } => {
                EventLogTypeSchema::ChatCreate { chat: chat.into() }
            }
            EventLogType::ChatMemberAdded { chat_id, member } => {
                EventLogTypeSchema::ChatMemberAdded {
                    chat_id: chat_id.to_string(),
                    member: member.into(),
                }
            }
            EventLogType::ChatMemberRemoved { chat_id, member } => {
                EventLogTypeSchema::ChatMemberRemoved {
                    chat_id: chat_id.to_string(),
                    member: member.into(),
                }
            }
        }
    }
}

pub async fn start_event_listener(
    jetstream: Context,
    event_log_repository: Arc<dyn EventLogRepository>,
    io: SocketIo,
) -> Result<(), anyhow::Error> {
    let consumer: consumer::PullConsumer = jetstream
        .create_consumer_on_stream(
            consumer::pull::Config {
                durable_name: Some("event_logger".to_string()),
                ..Default::default()
            },
            "events",
        )
        .await?;

    tokio::spawn(event_listener(consumer, event_log_repository, io));
    Ok(())
}

async fn event_listener(
    consumer: consumer::PullConsumer,
    event_log_repository: Arc<dyn EventLogRepository>,
    io: SocketIo,
) {
    loop {
        let stream = match consumer.messages().await {
            Ok(stream) => stream,
            Err(e) => {
                tracing::error!("Failed to fetch messages: {}", e);
                tokio::time::sleep(Duration::from_secs(1)).await;
                continue;
            }
        };

        let mut messages = stream.take(10);

        while let Some(event_result) = messages.next().await {
            let jetstream_msg = match event_result {
                Ok(msg) => msg,
                Err(e) => {
                    tracing::error!("Error receiving message: {}", e);
                    continue;
                }
            };

            let msg_id = jetstream_msg
                .headers
                .as_ref()
                .and_then(|h| h.get("Nats-Msg-Id"))
                .map(|v| v.to_string())
                .unwrap_or_else(|| "unknown".to_string());

            let event: Event = match serde_json::from_slice(&jetstream_msg.payload) {
                Ok(event) => event,
                Err(e) => {
                    tracing::error!("Failed to deserialize message {}: {}", msg_id, e);
                    let _ = jetstream_msg.ack().await;
                    continue;
                }
            };

            for &user_id in &event.recipients {
                let event_log = EventLog {
                    user_id,
                    event_id: event.event_id,
                    event_type: event.event_type.clone(),
                    created_at: event.created_at,
                };

                if let Err(e) = event_log_repository.save(event_log.clone()).await {
                    tracing::error!("Failed to save event log for user {}: {:#}", user_id, e);
                    continue;
                }

                io.to(format!("user:{}", user_id))
                    .emit("event", &EventLogSchema::from(event_log))
                    .await
                    .ok();
            }

            if let Err(e) = jetstream_msg.ack().await {
                tracing::error!("Failed to ack message {}: {}", msg_id, e);
            } else {
                tracing::info!("Processed and acked event: {}", event.event_id);
            }
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
