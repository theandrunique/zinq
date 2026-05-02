# Emojis and Stickers

## Stickers
- Size: 512x512px
- Static formats: `.png`, `.webp`

## Emoji
- Size: 100x100px
- Static formats: `.png`, `.webp`

## Animated
- Formats: `.webm` encoded with the VP9 codec
- Video duration: less than 3 seconds
- Frame rate: up to 60 FPS
- Max size: 256 KB
- Video must have no audio stream

## Tables

```cql
CREATE TABLE emoji_packs (
    pack_id: bigint,
    pack_name: text,
    display_name: text,
    owner_id: bigint,
    is_published: boolean,
    timestamp: timestamp,
    updated_timestamp: timestamp,
    preview_asset: text,

    PRIMARY KEY (pack_id)
);

CREATE TABLE emoji_pack_names (
    pack_name text,
    pack_id bigint,

    PRIMARY KEY (pack_name)
);

CREATE TABLE emojis (
    pack_id bigint,
    emoji_id text,
    shortcode text,
    asset text,
    order_index int,

    PRIMARY KEY (pack_id, emoji_id)
);
```

## REST API

### POST `/emoji-packs`

```json
{
    "display_name": "Cute Cats"
}
```

```cql
INSERT INTO emoji_packs (
    pack_id,
    display_name,
    owner_id,
    is_published,
    timestamp,
    preview_asset
) VALUES (?, ?, ?, ?, ?, ?);
```

### GET `/emoji-packs/{pack_id}`

```cql
SELECT * FROM emoji_packs WHERE pack_id = ?;

SELECT * FROM emojis WHERE pack_id = ?;
```

### PATCH `/emoji-packs/{pack_id}`

```cql
UPDATE emoji_packs
SET display_name = ?,
    preview_asset = ?
WHERE pack_id = ?;
```

### POST `/emoji-packs/{pack_id}/emojis`

```cql
SELECT order_index FROM emojis
WHERE pack_id = ?
ORDER BY order_index DESC
LIMIT 1;

INSERT INTO emojis (
    pack_id,
    emoji_id,
    shortcode,
    asset,
    order_index
) VALUES (?, ?, ?, ?, ?);
```

### DELETE `/emoji-packs/{pack_id}/emojis/{emoji_id}`

### PUT `/emoji-packs/{pack_id}/emojis/order`

```json
{
    "emoji_ids": ["em_1", "em_5", "em_3"]
}
```

```cql
BEGIN UNLOGGED BATCH
    UPDATE emoji SET order_index 1 WHERE pack_id = ? AND emoji_id = ?;
    UPDATE emoji SET order_index 2 WHERE pack_id = ? AND emoji_id = ?;
    UPDATE emoji SET order_index 3 WHERE pack_id = ? AND emoji_id = ?;
APPLY BATCH;
```

### POST `/emoji-packs/{pack_id}/publish`

```cql
SELECT pack_id FROM emoji_pack_names WHERE pack_name = ?;

INSERT INTO emoji_pack_names (pack_name, pack_id)
VALUES (?, ?) IF NOT EXISTS;

UPDATE emoji_packs
SET is_published = true,
    pack_name = ?,
    display_name = ?
WHERE pack_id = ?;
```

### DELETE `/emoji-packs/{pack_id}`

```cql
UPDATE emoji_packs
SET is_published = false,
    pack_name = null
WHERE pack_id = ?;
```
