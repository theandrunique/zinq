
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
    display_name: text,
    owner_id: bigint,
    is_published: text,
    timestamp: timestamp,
    preview_asset_url: text,
    PRIMARY KEY (pack_id)
);

CREATE TABLE emojis (
    pack_id bigint,
    emoji_id text,
    shortcode text,
    asset_url text,
    order int,
    PRIMARY KEY (pack_id, order)
) WITH CLUSTERING ORDER BY (order ASC);
```

## REST API

### POST /emoji-packs

```json
{
  "display_name": "Cute Cats"
}
```
*Or multipart for preview upload

```cql
INSERT INTO emoji_packs (pack_id, display_name, owner_id, is_published, timestamp, preview_asset)
VALUES (?, ?, ?, ?, ?, ?)
```

### GET /emoji-packs/{pack_id}

```cql
SELECT * FROM emoji_packs WHERE pack_id = ?;

SELECT * FROM emojis WHERE pack_id = ?;
```

### PATCH /emoji-packs/{pack_id}

multipart/form-data
- display_name
- preview_asset

```cql
UPDATE emoji_packs
SET display_name = ?
  AND preview_asset_url = ?
WHERE pack_id = ?
```

### POST /emoji-packs/{pack_id}/emojis

multipart/form-data
- emoji
- unicode_hint

```cql
SELECT order FROM emojis
WHERE pack_id = ? ORDER BY order DESC LIMIT 1;

INSERT INTO emojis (pack_id, emoji_id, unicode_hint, asset_url, order_index)
VALUES (?, ?, ?, ?, ?);
```

### DELETE /emoji-packs/{pack_id}/emojis/{emoji_id}

```cql
DELETE FROM emojis WHERE pack_id = ? AND emoji_id = ?;
```

### PUT /emoji-packs/{pack_id}/emojis/order

```cql
UPDATE pack_emojis SET order_index = ?
WHERE pack_id = ? AND emoji_id = ?;
```

### POST /emoji-packs/{pack_id}/publish

```cql
SELECT pack_id FROM emoji_pack_names WHERE pack_name = ?;

INSERT INTO emoji_pack_names (pack_name, pack_id) VALUES (?, ?);

UPDATE emoji_packs
SET is_published = true,
    pack_name = ?,
    display_name = ?,
    timestamp = now(),
WHERE pack_id = ?;
```

### DELETE /emoji-packs/{pack_id}
