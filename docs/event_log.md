# Event Log

## Code

### Entities

```rust
#[derive(Clone, Serialize, Deserialize)]
pub enum EventLogType {
    MessageCreate { message: Message },
    MessageUpdate { message: Message },
    MessageDelete { message_id: i64 },
    ChatCreate { chat: Chat },
}

#[derive(Serialize, Deserialize)]
pub struct EventLog {
    pub user_id: i64,
    pub event_id: i64,
    pub event_type: EventLogType,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
pub struct Event {
    pub event_id: i64,
    pub event_type: EventLogType,
    pub created_at: DateTime<Utc>,
    pub recipients: Vec<i64>,
}
```

### Data

```rust
#[async_trait]
pub trait EventLogRepository: Send + Sync {
    async fn save(&self, event: EventLog) -> Result<(), anyhow::Error>;

    async fn get_event_logs(
        &self,
        user_id: i64,
        after_event_id: i64,
        limit: i32,
    ) -> Result<Vec<EventLog>, anyhow::Error>;
}
```

## Tables

```cql
CREATE TABLE user_event_log (
    user_id bigint,
    event_id bigint,
    event_type text,
    timestamp timestamp,

    PRIMARY KEY ((user_id), event_id)
) WITH CLUSTERING ORDER BY (event_id ASC)
    AND compaction = {
        'class': 'TimeWindowCompactionStrategy',
        'compaction_window_unit': 'DAYS',
        'compaction_window_size': 1
    }
    AND default_time_to_live = 2592000; -- 30 days
```

## Client fast sync

- Request all the new logs from the last `event_id` client got
- Update the current state with new logs

## Client full sync

- Request current `event_id` from `user_event_log`
- Request all user chats and their last messages

## Chat Types

- DM
- GROUP

## Example use cases

### Write a message to `DM` / `GROUP` chat

- Command pipeline:

1. Send a request

2. Ensure idempotency

3. Insert message into the `messages_by_chat_id`

```cql
INSERT INTO messages_by_chat_id (
    chat_id,
    message_id,
    author_id,
    content,
    timestamp,
) VALUES (?, ?, ?, ?, ?)
```

4. Publish `Event` in `nats`

5. Return the result

- Worker pipeline:

1. Get `Event` from `nats`

2. For each recipient create a `user_event_log` record

```cql
INSERT INTO user_event_log (
    user_id,
    event_id,
    event_type,
    timestamp,
) VALUES (?, ?, ?, ?)
```

3. send `Event` to socket.io for every online member
