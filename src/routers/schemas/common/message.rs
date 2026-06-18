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
    pub msg_type: String,
    pub metadata: Option<MessageMetadataSchema>,
    pub attachments: Vec<AttachmentSchema>,
}

#[derive(Serialize, Debug)]
#[serde(untagged, rename_all = "snake_case")]
pub enum MessageMetadataSchema {
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

impl From<(Message, Vec<Attachment>)> for MessageSchema {
    fn from((message, attachments): (Message, Vec<Attachment>)) -> Self {
        let (msg_type, metadata) = match message.message_type {
            MessageType::Default => ("DEFAULT".to_string(), None),

            MessageType::Reply {
                referenced_message_id,
            } => (
                "REPLY".to_string(),
                Some(MessageMetadataSchema::Reply {
                    referenced_message_id: referenced_message_id.to_string(),
                }),
            ),

            MessageType::MemberAdd { user_id } => (
                "MEMBER_ADD".to_string(),
                Some(MessageMetadataSchema::MemberAdd {
                    user_id: user_id.to_string(),
                }),
            ),

            MessageType::MemberRemove { user_id } => (
                "MEMBER_REMOVE".to_string(),
                Some(MessageMetadataSchema::MemberRemove {
                    user_id: user_id.to_string(),
                }),
            ),

            MessageType::MemberLeave { user_id } => (
                "MEMBER_LEAVE".to_string(),
                Some(MessageMetadataSchema::MemberLeave {
                    user_id: user_id.to_string(),
                }),
            ),

            MessageType::ChatNameUpdate { new_name } => (
                "CHAT_NAME_UPDATE".to_string(),
                Some(MessageMetadataSchema::ChatNameUpdate { new_name }),
            ),

            MessageType::ChatImageUpdate { new_image } => (
                "CHAT_IMAGE_UPDATE".to_string(),
                Some(MessageMetadataSchema::ChatImageUpdate { new_image }),
            ),

            MessageType::ChatPinnedMessage => ("CHAT_PINNED_MESSAGE".to_string(), None),
            MessageType::ChatUnpinMessage => ("CHAT_UNPIN_MESSAGE".to_string(), None),

            MessageType::ChatCreate { chat_name } => (
                "CHAT_CREATE".to_string(),
                Some(MessageMetadataSchema::ChatCreate { chat_name }),
            ),

            MessageType::Forward => ("FORWARD".to_string(), None),
        };

        Self {
            id: message.id.to_string(),
            chat_id: message.chat_id.to_string(),
            author_id: message.author_id.to_string(),
            content: message.content,
            created_at: message.created_at,
            edited_at: message.edited_at,
            msg_type,
            metadata,
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
                let (msg_type, metadata) = match message.message_type {
                    MessageType::Default => ("DEFAULT".to_string(), None),
                    MessageType::Reply {
                        referenced_message_id,
                    } => (
                        "REPLY".to_string(),
                        Some(MessageMetadataSchema::Reply {
                            referenced_message_id: referenced_message_id.to_string(),
                        }),
                    ),
                    MessageType::MemberAdd { user_id } => (
                        "MEMBER_ADD".to_string(),
                        Some(MessageMetadataSchema::MemberAdd {
                            user_id: user_id.to_string(),
                        }),
                    ),
                    MessageType::MemberRemove { user_id } => (
                        "MEMBER_REMOVE".to_string(),
                        Some(MessageMetadataSchema::MemberRemove {
                            user_id: user_id.to_string(),
                        }),
                    ),
                    MessageType::MemberLeave { user_id } => (
                        "MEMBER_LEAVE".to_string(),
                        Some(MessageMetadataSchema::MemberLeave {
                            user_id: user_id.to_string(),
                        }),
                    ),
                    MessageType::ChatNameUpdate { new_name } => (
                        "CHAT_NAME_UPDATE".to_string(),
                        Some(MessageMetadataSchema::ChatNameUpdate { new_name }),
                    ),
                    MessageType::ChatImageUpdate { new_image } => (
                        "CHAT_IMAGE_UPDATE".to_string(),
                        Some(MessageMetadataSchema::ChatImageUpdate { new_image }),
                    ),
                    MessageType::ChatPinnedMessage => ("CHAT_PINNED_MESSAGE".to_string(), None),
                    MessageType::ChatUnpinMessage => ("CHAT_UNPIN_MESSAGE".to_string(), None),
                    MessageType::ChatCreate { chat_name } => (
                        "CHAT_CREATE".to_string(),
                        Some(MessageMetadataSchema::ChatCreate { chat_name }),
                    ),
                    MessageType::Forward => ("FORWARD".to_string(), None),
                };

                MessageSchema {
                    id: message.id.to_string(),
                    chat_id: message.chat_id.to_string(),
                    author_id: message.author_id.to_string(),
                    content: message.content,
                    created_at: message.created_at,
                    edited_at: message.edited_at,
                    msg_type,
                    metadata,
                    attachments: attachments.into_iter().map(|a| a.into()).collect(),
                }
            })
            .collect()
    }
}
