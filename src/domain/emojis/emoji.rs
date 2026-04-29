use chrono::{DateTime, Utc};

pub struct EmojiPack {
    pub id: i64,
    pub pack_name: String,
    pub display_name: String,
    pub owner_id: i64,
    pub is_published: bool,
    pub timestamp: DateTime<Utc>,
    pub updated_timestamp: DateTime<Utc>,
    pub preview_asset: String,
}

pub struct Emoji {
    pub id: String,
    pub pack_id: i64,
    pub shortcode: Option<String>,
    pub asset: String,
    pub order: i64,
}
