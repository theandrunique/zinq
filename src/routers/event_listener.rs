use std::sync::Arc;
use std::time::Duration;

use async_nats::jetstream::{Context, consumer};
use chrono::{DateTime, Utc};
use futures_util::StreamExt;
use serde::Serialize;
use socketioxide::SocketIo;

use crate::{
    domain::event_log::{Event, EventLog, EventLogType, data::EventLogRepository},
    routers::schemas::common::EventLogSchema,
};

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
        let mut stream = match consumer.messages().await {
            Ok(stream) => stream,
            Err(e) => {
                tracing::error!("Failed to get events stream: {}", e);
                tokio::time::sleep(Duration::from_secs(5)).await;
                continue;
            }
        };

        while let Some(stream_message_result) = stream.next().await {
            let stream_message = match stream_message_result {
                Ok(m) => m,
                Err(e) => {
                    tracing::error!("Error receiving message: {}", e);
                    continue;
                }
            };

            let msg_id = stream_message
                .headers
                .as_ref()
                .and_then(|h| h.get("Nats-Msg-Id"))
                .map(|v| v.to_string())
                .unwrap_or_else(|| "unknown".to_string());

            let event: Event = match serde_json::from_slice(&stream_message.payload) {
                Ok(event) => event,
                Err(e) => {
                    tracing::error!("Failed to deserialize message {}: {}", msg_id, e);
                    let _ = stream_message.ack().await;
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

                if let Err(e) = event_log_repository.save(&event_log).await {
                    tracing::error!("Failed to save event log for user {}: {:#}", user_id, e);
                    continue;
                }

                if let Err(e) = io
                    .to(format!("user:{}", user_id))
                    .emit("event", &EventLogSchema::from(event_log))
                    .await
                {
                    tracing::error!("Failed to emit event to user {}: {}", user_id, e);
                    continue;
                }
            }

            if let Err(e) = stream_message.ack().await {
                tracing::error!("Failed to ack message {}: {}", msg_id, e);
            } else {
                tracing::info!("Processed and acked event: {}", event.event_id);
            }
        }

        tracing::warn!("Event stream closed, reconnecting...");
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
