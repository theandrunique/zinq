# Message Acks

## Tables

```cql
CREATE TABLE chat_users_by_user_id (
    user_id bigint,
    chat_id bigint,
    last_read_message_id bigint,
    -- ...

    PRIMARY KEY ((user_id), chat_id)
);

CREATE TABLE message_acks (
    chat_id bigint,
    message_id bigint,
    user_id bigint,
    timestamp timestamp,

    PRIMARY KEY ((chat_id), message_id, user_id)
) WITH CLUSTERING ORDER BY (message_id DESC)
    AND default_time_to_live = 604800; -- 7 days

CREATE TABLE message_views (
    chat_id bigint,
    message_id bigint,
    views counter,

    PRIMARY KEY ((chat_id), message_id)
);
```

## REST API

### `GROUP` / `DM`

- PUT `/chats/{chat_id}/messages/{message_id}/ack`

```cql
-- если текущий меньше нового
UPDATE chat_users_by_user_id
SET last_read_message_id = ?
WHERE user_id = ? AND chat_id = ?;

-- определить сообщения за последние 7 дней, которые еще не были прочитаны и обновить их прочтения
INSERT INTO message_acks (
    chat_id,
    message_id,
    user_id,
    timestamp
) VALUES (?, ?, ?, ?);
```

- GET `/chats/{chat_id}/messages/{message_id}/acks`

```cql
SELECT * FROM message_acks WHERE chat_id = ? AND message_id = ?;
```

### `SUPER_GROUP`

- PUT `/chats/{chat_id}/messages/{message_id}/ack`

```cql
-- если текущий меньше нового
UPDATE chat_users_by_user_id
SET last_read_message_id = ?
WHERE user_id = ? AND chat_id = ?;
```

### `CHANNEL`

- POST `/chats/{chat_id}/messages/acks`

```json
{
    "message_ids": ["101", "102", "103"]
}
```

```cql
-- если текущий меньше нового
UPDATE chat_users_by_user_id
SET last_read_message_id = ?
WHERE user_id = ? AND chat_id = ?;

BEGIN UNLOGGED BATCH
    UPDATE message_views
    SET views = views + 1
    WHERE chat_id = ? AND message_id = ?;
APPLY BATCH;
```
