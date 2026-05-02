# Reactions

## Tables

```cql
CREATE TABLE reactions_by_chat_id (
    chat_id: bigint,
    message_id: bigint,
    user_id: biging,
    pack_id: string,
    emoji_id: string,
    timestamp: timestamp

    PRIMARY KEY ((chat_id, message_id), user_id, pack_id, emoji_id)
);

CREATE TABLE reaction_counts_by_chat_id (
    chat_id: bigint,
    message_id: bigint,
    pack_id: string,
    emoji_id: string,
    count: counter

    PRIMARY KEY ((chat_id, message_id), pack_id, emoji_id)
);
```

## REST API

### PUT `/chats/{chat_id}/messages/{message_id}/reactions`

```json
{
    "pack_id": 0,
    "emoji_id": "fire"
}
```

```cql
INSERT INTO reactions_by_chat_id (
    chat_id,
    message_id,
    user_id,
    pack_id,
    emoji_id,
    timestamp
) VALUES (?, ?, ?, ?, ?, ?)
IF NOT EXISTS;

-- increment the counter if reaction is new

UPDATE reaction_counts_by_chat_id
SET count = count + 1
WHERE chat_id = ?
    AND message_id = ?
    AND pack_id = ?
    AND emoji_id = ?;
```

### DELETE `/chats/{chat_id}/messages/{message_id}/reactions`

```json
{
    "pack_id": 0,
    "emoji_id": "fire"
}
```

```cql
DELETE reactions_by_chat_id
WHERE chat_id = ?
    AND message_id = ?
    AND user_id = ?
    AND pack_id = ?
    AND emoji_id = ?;

-- decrement the counter if reaction was existing

UPDATE reaction_counts_by_chat_id
SET count = count - 1
WHERE chat_id = ?
    AND message_id = ?
    AND pack_id = ?
    AND emoji_id = ?;
```

### GET `/chats/{chat_id}/messages/{message_id}/reactions`

- desc: Detailed view of user reactions

```cql
SELECT * FROM reactions_by_chat_id
WHERE chat_id = ? AND message_id = ?;
```

Response

```json
[
    {
        "pack_id": 0,
        "emoji_id": "fire",
        "total_count": 4,
        "users": [
            {"user_id": 1001, "timestamp": "2025-03-15T10:30:00Z"}
            {"user_id": 1002, "timestamp": "2025-03-15T10:31:00Z"}
            // ...
        ]
    },
    {
        "pack_id": 0,
        "emoji_id": "smile",
        "total_count": 7,
        "users": [
            {"user_id": 1003, "timestamp": "..."}
            // ...
        ]
    }
]
```
