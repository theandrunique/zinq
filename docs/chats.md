# Chats

## Tables

```cql
CREATE TABLE chats_by_id (
    chat_id bigint,
    type int,
    name text,
    description text,
    owner_id bigint,
    image text,
    last_message_id bigint,
    permission_overrides bigint,
    timestamp timestamp,

    PRIMARY KEY (chat_id)
);

CREATE TABLE chat_users_by_user_id (
    user_id bigint,
    chat_id bigint,
    last_read_message_id bigint,
    username text,
    global_name text,
    image text,
    permission_overwrites bigint,
    is_leave boolean,

    PRIMARY KEY (user_id, chat_id)
);

CREATE MATERIALIZED VIEW chat_users_by_chat_id AS
    SELECT
        user_id,
        chat_id,
        last_read_message_id,
        username,
        global_name,
        image,
        permission_overwrites,
        is_leave
    FROM chat_users_by_user_id
    WHERE
        chat_id IS NOT NULL
        AND user_id IS NOT NULL
    PRIMARY KEY (chat_id, user_id);

CREATE TABLE private_chats (
    user_id1 bigint,
    user_id2 bigint,
    chat_id bigint,

    PRIMARY KEY ((user_id1, user_id2))
);
```

## REST API

### POST /chats

```json
{
    "name": "Some Cool Name",
    "members": ["u_1", "u_2", "u_3"]
}
```

```cql
INSERT INTO chats_by_id (
    chat_id,
    type,
    name,
    description,
    owner_id,
    image,
    last_message_id,
    permission_overrides,
    timestamp
) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?);

BEGIN UNLOGGED BATCH
    INSERT INTO chat_users_by_user_id (
        user_id,
        chat_id,
        last_read_message_id,
        username,
        global_name,
        image,
        permission_overwrites,
        is_leave
    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?);
    -- and so on
APPLY BATCH;
```

### GET /chats/{chat_id}

```cql
SELECT * FROM chats_by_id WHERE chat_id = ?;
```

### PATCH /chats/{chat_id}

```cql
UPDATE chats_by_id
SET name = ?,
    image = ?,
    description = ?
WHERE chat_id = ?;
```

### GET /users/@me/chats

```cql
SELECT * FROM chat_users_by_user_id WHERE user_id = ?;
```

### GET /users/@me/dms/me

```cql
SELECT chat_id FROM private_chats
WHERE user_id1 = ? AND user_id2 = ?;

SELECT * FROM chats_by_id WHERE chat_id = ?;
```

### GET /users/@me/dms/{user_id}

```cql
SELECT chat_id FROM private_chats
WHERE user_id1 = ? AND user_id2 = ?;

SELECT * FROM chats_by_id WHERE chat_id = ?;
```

### PUT /chats/{chat_id}/members/{user_id}

```cql
SELECT * FROM chats_by_id WHERE chat_id = ?;

INSERT INTO chat_users_by_user_id (
    user_id,
    chat_id,
    last_read_message_id,
    username,
    global_name,
    image,
    permission_overwrites,
    is_leave
) VALUES (?, ?, ?, ?, ?, ?, ?, ?);
```

### DELETE /chats/{chat_id}/members/{user_id}

```cql
SELECT * FROM chats_by_id WHERE chat_id = ?;

UPDATE chat_users_by_user_id
SET is_leave = true
WHERE user_id = ?;
```
