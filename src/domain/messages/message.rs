use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum MessageType {
    Default,
    Reply,
    MemberAdd,
    MemberRemove,
    MemberLeave,
    ChannelNameUpdate,
    ChannelImageUpdate,
    ChannelPinnedMessage,
    ChannelUnpinMessage,
    ChannelCreate,
    Forward,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum MessageMetadata {
    None,
}

pub struct Message {
    pub id: i64,
    pub channel_id: i64,
    pub author_id: i64,
    pub target_user_id: Option<i64>,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub edited_timestamp: DateTime<Utc>,
    pub message_type: MessageType,
    pub referenced_message_id: Option<i64>,
    pub metadata: MessageMetadata,
}
