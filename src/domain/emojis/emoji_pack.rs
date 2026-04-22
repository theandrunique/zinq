use chrono::{DateTime, Utc};

pub struct EmojiPack {
    pub id: i64,
    pub owner_id: i64,
    pub display_name: String,
    pub is_published: bool,
    pub timestamp: DateTime<Utc>,
    pub preview_asset_url: String,
}
