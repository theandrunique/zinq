# Event Log

## Event Types

- `MESSAGE_CREATE`
- `MESSAGE_UPDATE`
- `MESSAGE_DELETE`
- `MESSAGE_REACTION` (only for DM/GROUP)
- `CHANNEL_INFO_UPDATE`

## Tables

```cql
CREATE TABLE user_event_log (
    user_id bigint,
    event_id bigint,
    event_type int,
    event_data blob,
    server_timestamp timestamp,
    chat_id bigint,
    message_id bigint,
    PRIMARY KEY ((user_id), event_id)
) WITH CLUSTERING ORDER BY (event_id ASC)
    AND compaction = {
        'class': 'TimeWindowCompactionStrategy',
        'compaction_window_unit': 'DAYS',
        'compaction_window_size': 1
    }
    AND default_time_to_live = 2592000; -- 30 days

CREATE TABLE chat_event_log (
    chat_id bigint,
    event_id bigint,
    event_type int,
    event_data blob,
    server_timestamp timestamp,
    message_id bigint,
    user_id bigint,
    PRIMARY KEY ((chat_id), event_id)
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
- Request updates for `SUPER_GROUPS`

## Client full sync

- Request current `event_id` from `user_event_log`
- Request all user chats
- Request `event_id` for all the chats type of `SUPER_GROUP`

## Chat Types

- DM
- GROUP
- SUPER_GROUP

## Example use cases

### Write a message to `DM` / `GROUP` chat

1. Send a request

2. Insert message into the `messages_by_chat_id`

```cql
INSERT INTO messages_by_chat_id (
    chat_id,
    message_id,
    author_id,
    content,
    timestamp,
) VALUES (?, ?, ?, ?, ?)
```

3. Create record into each chat user event log `user_event_log`

```cql
INSERT INTO user_event_log (
    user_id,
    event_id,
    event_type,
    event_data,
    server_timestamp,
    chat_id,
    message_id,
) VALUES (?, ?, ?, ?, ?, ?, ?)
```

4. send MESSAGE_CREATE to socket.io for every online member

### Write a message to `SUPER_GROUP` chat

1. Send a request

2. Insert message into the `messages_by_chat_id`

```cql
INSERT INTO messages_by_chat_id (
    chat_id,
    message_id,
    author_id,
    content,
    timestamp,
) VALUES (?, ?, ?, ?, ?)
```

3. Create one record into the event log `chat_event_log`

```cql
INSERT INTO chat_event_log (
    chat_id,
    event_id,
    event_type,
    event_data,
    server_timestamp,
    message_id,
) VALUES (?, ?, ?, ?, ?, ?, ?)
```

4. send MESSAGE_CREATE to socket.io for every online member
