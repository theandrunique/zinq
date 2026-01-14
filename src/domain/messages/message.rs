use chrono::{DateTime, Utc};

use crate::domain::messages::attachment::Attachment;

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

pub struct MessageAuthor {
    pub id: i64,
    pub username: String,
    pub global_name: String,
    pub avatar: String,
}

pub struct Message {
    pub id: i64,
    pub channel_id: i64,
    pub author_id: i64,
    pub target_user_id: Option<i64>,
    pub author: MessageAuthor,
    pub target_user: Option<MessageAuthor>,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub edited_timestamp: DateTime<Utc>,
    pub attachments: Vec<Attachment>,
    pub message_type: MessageType,
    pub referenced_message_id: Option<i64>,
}
