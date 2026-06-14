# Idempotency

## Tables

```cql
CREATE TABLE command_idempotency (
    user_id bigint,
    idempotency_key uuid,
    event_id bigint,

    PRIMARY KEY (user_id, idempotency_key)
) AND default_time_to_live = 600;
```

## Logic

- User sends idempotency key with for each command

- Execute

```cql
INSERT INTO command_idempotency (user_id, idempotency_key, event_id)
VALUES (?, ?, ?) IF NOT EXISTS;
```

- If applied == false, select the event_id

- Restore the result and return it

- If applied == true, execute the command and store the result
