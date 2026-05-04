CREATE KEYSPACE IF NOT EXISTS zinq WITH replication = {'class': 'SimpleStrategy', 'replication_factor': 1};

use zinq;

CREATE TABLE attachments_by_message_id (
    chat_id bigint,
    attachment_id bigint,
    message_id bigint,
    content_type text,
    duration_secs float,
    filename text,
    is_spoiler boolean,
    placeholder text,
    signed_url text,
    signed_url_expires timestamp,
    size bigint,
    waveform text,
    timestamp timestamp,

    PRIMARY KEY ((chat_id), message_id, attachment_id)
);

CREATE MATERIALIZED VIEW attachments_by_id AS
    SELECT
        chat_id,
        attachment_id,
        message_id,
        content_type,
        duration_secs,
        filename,
        is_spoiler,
        placeholder,
        signed_url,
        signed_url_expires,
        size,
        waveform,
        timestamp
    FROM attachments_by_message_id
    WHERE
        chat_id IS NOT NULL
        AND attachment_id IS NOT NULL
        AND message_id IS NOT NULL
    PRIMARY KEY ((chat_id), attachment_id, message_id)
    WITH CLUSTERING ORDER BY (attachment_id DESC);

CREATE TABLE chat_users_by_user_id (
    user_id bigint,
    chat_id bigint,
    last_read_message_id bigint,
    username text,
    global_name text,
    image text,
    permission_overwrites bigint,
    is_leave boolean,

    PRIMARY KEY ((user_id), chat_id)
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
    PRIMARY KEY ((chat_id), user_id);

CREATE TABLE chats_by_id (
    chat_id bigint,
    type int,
    name text,
    owner_id bigint,
    image text,
    last_message_id bigint,
    permissions bigint,
    timestamp timestamp,

    PRIMARY KEY ((chat_id))
);

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

CREATE TABLE message_acks (
    chat_id bigint,
    user_id bigint,
    to_message_id bigint,
    from_message_id bigint,
    timestamp timestamp,

    PRIMARY KEY ((chat_id), to_message_id)
) WITH CLUSTERING ORDER BY (to_message_id DESC);

CREATE TABLE private_chats (
    user_id1 bigint,
    user_id2 bigint,
    chat_id bigint,

    PRIMARY KEY ((user_id1, user_id2))
);

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
    PRIMARY KEY ((chat_id), event_id)
) WITH CLUSTERING ORDER BY (event_id ASC)
    AND compaction = {
        'class': 'TimeWindowCompactionStrategy',
        'compaction_window_unit': 'DAYS',
        'compaction_window_size': 1
    }
    AND default_time_to_live = 2592000; -- 30 days

CREATE TABLE sessions (
    user_id bigint,
    session_id bigint,
    client_name text,
    timestamp timestamp,
    device_name text,
    last_used_timestamp timestamp,
    location text,
    token_id uuid,

    PRIMARY KEY ((user_id), session_id)
);

CREATE INDEX sessions_token_id_idx ON sessions(token_id);


CREATE TABLE users (
    user_id bigint,
    bio text,
    timestamp timestamp,
    sessions_lifetime text,
    email text,
    email_updated_timestamp timestamp,
    global_name text,
    is_active boolean,
    is_email_verified boolean,
    totp_key blob,
    password_hash text,
    password_updated_timestamp timestamp,
    mfa boolean,
    username text,
    username_updated_timestamp timestamp,
    avatar text,

    PRIMARY KEY ((user_id))
);


CREATE TABLE users_by_username (
    username text,
    user_id bigint,

    PRIMARY KEY ((username))
);

CREATE TABLE users_by_email (
    email text,
    user_id bigint,

    PRIMARY KEY ((email))
);


CREATE TABLE verification_codes (
    identifier text, -- email, phone, etc
    scenario int,   -- (enum) reset_password, verify_email, etc
    code_hash text,
    timestamp timestamp,
    expires_timestamp timestamp,
    attempts int,
    PRIMARY KEY ((identifier, scenario))
);
