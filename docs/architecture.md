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
    AND default_time_to_live = 2592000; -- 30 дней

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
    AND default_time_to_live = 2592000; -- 30 дней
```

# Use cases

## Write a DM message

1. Клиент отправляет запрос на /chats/{chat_id}/messages
```json
{
  "content": "Hi!"
}
```
2. Проверки валидности запроса
3. Создается запись в таблице `messages`
```cql
INSERT INTO messages (
    chat_id,
    message_id,
    author_id,
    content,
    timestamp,
) VALUES (?, ?, ?, ?, ?)
```
4. Создаются записи на каждого участника чата в `user_event_log`
```cql
INSERT INTO user_event_log (
    user_id,
    event_id,
    event_type,
    event_data,
    server_timestamp,
    chat_id,
    message_id bigint,
) VALUES (?, ?, ?, ?, ?, ?, ?)
```
5. Клиенту отправляется ответ 200
6. По вебсокетам отправляются эвенты MESSAGE_CREATE для каждого участника чата, они доходят только до онлайн клиентов

## Client fast sync

## Client full sync


# Reactions

```cql
CREATE TABLE reactions_by_message_id (
    chat_id: bigint,
    message_id: bigint,
    user_id: biging,
    pack_id: string,
    emoji: string,
    timestamp: timestamp

    PRIMARY KEY ((chat_id, message_id), user_id, pack_id, emoji)
);

CREATE TABLE reaction_counts_by_message_id (
    chat_id: bigint,
    message_id: bigint,
    pack_id: string,
    emoji: string,
    count: counter

    PRIMARY KEY ((chat_id, message_id), pack_id, emoji)
);
```
