# Messages

## Tables

```cql
CREATE TABLE messages_by_chat_id (
    chat_id bigint,
    message_id bigint,
    author_id bigint,
    target_user_id bigint,
    content text,
    timestamp timestamp,
    edited_timestamp timestamp,
    pinned boolean,
    type int,
    referenced_message_id bigint,
    metadata text,

    PRIMARY KEY ((chat_id), message_id)
) WITH CLUSTERING ORDER BY (message_id DESC);

CREATE TABLE attachments_by_chat_id (
    chat_id bigint,
    message_id bigint,
    attachment_id bigint,
    content_type text,
    duration_secs float,
    filename text,
    is_spoiler boolean,
    placeholder text,
    size bigint,
    waveform text,
    timestamp timestamp,

    PRIMARY KEY ((chat_id), message_id, attachment_id)
);
```

## How to upload attachments

- Request POST `/chats/{chat_id}/attachments`

Request:

```json
{
    "files": [
        {
            "id": "any optional id here",
            "filename": "filename_for_object_storage",
            "file_size": 32441
        }
    ]
}
```

Response:

```json
{
    "results": [
        {
            "id": "your id if you provided any",
            "upload_url": "signed url for upload",
            "upload_filename": "attachments/{chat_id}/{attachment_id}/{filename}"
        }
    ],
    "errors": [
        {
            "id": "your id if you provided any",
            "errors": ["File is too large."],
        }
    ]
}
```

- Upload the files with the `upload_url` you got from the server
- Use the `upload_filename` in create message request
- Done!


## REST API

### POST `/chats/{chat_id}/messages`

```json
{
    "content": "Some cool message here",
    "referenced_message_id": "342342",
    attachments: [
        {
            "uploaded_filename": "attachments/{chat_id}/{attachment_id}/{filename}",
            "filename": "some_music.mp3"
        }
    ]
}
```

```cql
INSERT INTO messages_by_chat_id (
    chat_id,
    message_id,
    author_id,
    target_user_id,
    content,
    timestamp,
    edited_timestamp,
    pinned,
    type,
    referenced_message_id,
    metadata
) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?);


BEGIN UNLOGGED BATCH
    INSERT INTO attachments_by_chat_id (
        chat_id,
        message_id,
        attachment_id,
        content_type,
        duration_secs,
        filename,
        is_spoiler,
        placeholder,
        size,
        waveform,
        timestamp
    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?);
APPLY BATCH;
```

### PUT `/chats/{chat_id}/messages/{message_id}`

```json
{
    "content": "Some cool updated message here",
    "referenced_message_id": "342342",
    attachments: [
        {
            "uploaded_filename": "attachments/{chat_id}/{attachment_id}/{filename}",
            "filename": "some_music.mp3"
        }
    ]
}
```

### GET `/chats/{chat_id}/messages/{message_id}?before=1234&limit=50`

```cql
SELECT * FROM message_by_chat_id
WHERE chat_id = ? AND message_id < ?
LIMIT ?;
```

### DELETE `/chats/{chat_id}/messages/{message_id}`

```cql
DELETE FROM messages_by_chat_id WHERE chat_id = ? AND message_id = ?;

DELETE FROM attachments_by_chat_id WHERE chat_id = ? AND message_id = ?;
```
