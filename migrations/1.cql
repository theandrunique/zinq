CREATE TABLE IF NOT EXISTS attachments_by_message_id (
    chat_id bigint,
    message_id bigint,
    attachment_id bigint,
    content_type text,
    duration_secs float,
    filename text,
    is_spoiler boolean,
    placeholder text,
    storage_key text,
    size bigint,
    waveform text,
    timestamp timestamp,

    PRIMARY KEY ((chat_id), message_id, attachment_id)
);

CREATE MATERIALIZED VIEW IF NOT EXISTS attachments_by_id AS
    SELECT
        chat_id,
        attachment_id,
        message_id,
        content_type,
        duration_secs,
        filename,
        is_spoiler,
        placeholder,
        storage_key,
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

CREATE TABLE IF NOT EXISTS chat_users_by_user_id (
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

CREATE MATERIALIZED VIEW IF NOT EXISTS chat_users_by_chat_id AS
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

CREATE TABLE IF NOT EXISTS chats_by_id (
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

CREATE TABLE IF NOT EXISTS messages_by_chat_id (
    chat_id bigint,
    message_id bigint,
    author_id bigint,
    content text,
    timestamp timestamp,
    edited_timestamp timestamp,
    type text,

    PRIMARY KEY ((chat_id), message_id)
) WITH CLUSTERING ORDER BY (message_id DESC);

CREATE TABLE IF NOT EXISTS message_acks (
    chat_id bigint,
    user_id bigint,
    to_message_id bigint,
    from_message_id bigint,
    timestamp timestamp,

    PRIMARY KEY ((chat_id), to_message_id)
) WITH CLUSTERING ORDER BY (to_message_id DESC);

CREATE TABLE IF NOT EXISTS private_chats (
    user_id1 bigint,
    user_id2 bigint,
    chat_id bigint,

    PRIMARY KEY ((user_id1, user_id2))
);

CREATE TABLE IF NOT EXISTS user_event_log (
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

CREATE TABLE IF NOT EXISTS chat_event_log (
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

CREATE TABLE IF NOT EXISTS sessions (
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

CREATE INDEX IF NOT EXISTS sessions_token_id_idx ON sessions(token_id);


CREATE TABLE IF NOT EXISTS users (
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


CREATE TABLE IF NOT EXISTS users_by_username (
    username text,
    user_id bigint,

    PRIMARY KEY ((username))
);

CREATE TABLE IF NOT EXISTS users_by_email (
    email text,
    user_id bigint,

    PRIMARY KEY ((email))
);


CREATE TABLE IF NOT EXISTS verification_codes (
    identifier text, -- email, phone, etc
    scenario int,   -- (enum) reset_password, verify_email, etc
    code_hash text,
    timestamp timestamp,
    expires_timestamp timestamp,
    attempts int,
    PRIMARY KEY ((identifier, scenario))
);
