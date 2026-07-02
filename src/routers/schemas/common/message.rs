use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::{
    application::messages::GetMessagesQueryResult,
    domain::{
        attachments::Attachment,
        messages::{Message, MessageType},
    },
};

#[derive(Serialize)]
pub struct AttachmentSchema {
    pub id: String,
    pub message_id: String,
    pub chat_id: String,
    pub filename: String,
    pub content_type: String,
    pub size: i64,
    pub storage_key: String,
    pub created_at: DateTime<Utc>,
}

impl From<Attachment> for AttachmentSchema {
    fn from(value: Attachment) -> Self {
        Self {
            id: value.id.to_string(),
            message_id: value.message_id.to_string(),
            chat_id: value.chat_id.to_string(),
            filename: value.filename,
            content_type: value.content_type,
            size: value.size,
            storage_key: value.storage_key,
            created_at: value.created_at,
        }
    }
}

#[derive(Serialize)]
pub struct MessageSchema {
    pub id: String,
    pub chat_id: String,
    pub author_id: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub edited_at: Option<DateTime<Utc>>,
    #[serde(rename = "type")]
    pub message_type: MessageTypeSchema,
    pub attachments: Vec<AttachmentSchema>,
}

#[derive(Serialize, Debug)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum MessageTypeSchema {
    Default,
    Reply { referenced_message_id: String },
    MemberAdd { user_id: String },
    MemberRemove { user_id: String },
    MemberLeave { user_id: String },
    ChatNameUpdate { new_name: String },
    ChatImageUpdate { new_image: String },
    ChatPinnedMessage,
    ChatUnpinMessage,
    ChatCreate { chat_name: String },
    Forward,
}

impl From<MessageType> for MessageTypeSchema {
    fn from(value: MessageType) -> Self {
        match value {
            MessageType::Default => MessageTypeSchema::Default,
            MessageType::Reply {
                referenced_message_id,
            } => MessageTypeSchema::Reply {
                referenced_message_id: referenced_message_id.to_string(),
            },
            MessageType::MemberAdd { user_id } => MessageTypeSchema::MemberAdd {
                user_id: user_id.to_string(),
            },
            MessageType::MemberRemove { user_id } => MessageTypeSchema::MemberRemove {
                user_id: user_id.to_string(),
            },
            MessageType::MemberLeave { user_id } => MessageTypeSchema::MemberLeave {
                user_id: user_id.to_string(),
            },
            MessageType::ChatNameUpdate { new_name } => {
                MessageTypeSchema::ChatNameUpdate { new_name }
            }
            MessageType::ChatImageUpdate { new_image } => {
                MessageTypeSchema::ChatImageUpdate { new_image }
            }
            MessageType::ChatPinnedMessage => MessageTypeSchema::ChatPinnedMessage,
            MessageType::ChatUnpinMessage => MessageTypeSchema::ChatUnpinMessage,
            MessageType::ChatCreate { chat_name } => MessageTypeSchema::ChatCreate { chat_name },
            MessageType::Forward => MessageTypeSchema::Forward,
        }
    }
}

impl From<(Message, Vec<Attachment>)> for MessageSchema {
    fn from((message, attachments): (Message, Vec<Attachment>)) -> Self {
        Self {
            id: message.id.to_string(),
            chat_id: message.chat_id.to_string(),
            author_id: message.author_id.to_string(),
            content: message.content,
            created_at: message.created_at,
            edited_at: message.edited_at,
            message_type: message.message_type.into(),
            attachments: attachments.into_iter().map(|a| a.into()).collect(),
        }
    }
}

impl From<GetMessagesQueryResult> for Vec<MessageSchema> {
    fn from(result: GetMessagesQueryResult) -> Self {
        result
            .messages
            .into_iter()
            .map(|message| {
                let attachments = result
                    .attachments
                    .get(&message.id)
                    .cloned()
                    .unwrap_or_default();

                MessageSchema {
                    id: message.id.to_string(),
                    chat_id: message.chat_id.to_string(),
                    author_id: message.author_id.to_string(),
                    content: message.content,
                    created_at: message.created_at,
                    edited_at: message.edited_at,
                    message_type: message.message_type.into(),
                    attachments: attachments.into_iter().map(|a| a.into()).collect(),
                }
            })
            .collect()
    }
}

impl From<Message> for MessageSchema {
    fn from(val: Message) -> Self {
        MessageSchema {
            id: val.id.to_string(),
            chat_id: val.chat_id.to_string(),
            author_id: val.author_id.to_string(),
            content: val.content,
            created_at: val.created_at,
            edited_at: val.edited_at,
            message_type: val.message_type.into(),
            attachments: vec![],
        }
    }
}
