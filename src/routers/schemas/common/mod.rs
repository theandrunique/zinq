use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::{
    application::messages::{AddOrEditMessageCommandResult, GetMessagesQueryResult},
    domain::{
        attachments::Attachment,
        auth::{SessionLifetime, User},
        chats::{Chat, ChatPermissions, ChatType},
        messages::{Message, MessageType},
    },
};

#[derive(Serialize)]
pub struct UserPrivateSchema {
    pub id: String,
    pub username: String,
    pub global_name: String,
    pub bio: Option<String>,
    pub avatar: Option<String>,
    pub timestamp: DateTime<Utc>,

    pub sessions_lifetime: SessionLifetime,
    pub mfa: bool,
    pub email: String,
    pub is_email_verified: bool,
}

impl From<User> for UserPrivateSchema {
    fn from(value: User) -> Self {
        Self {
            id: value.id.to_string(),
            username: value.username,
            global_name: value.display_name,
            bio: value.bio,
            avatar: value.avatar,
            timestamp: value.created_at,
            sessions_lifetime: value.sessions_lifetime,
            mfa: value.mfa,
            email: value.email,
            is_email_verified: value.is_email_verified,
        }
    }
}

#[derive(Serialize)]
pub struct UserPublicSchema {
    pub id: String,
    pub username: String,
    pub global_name: String,
    pub bio: Option<String>,
    pub avatar: Option<String>,
    pub timestamp: DateTime<Utc>,
}

impl From<User> for UserPublicSchema {
    fn from(value: User) -> Self {
        Self {
            id: value.id.to_string(),
            username: value.username,
            global_name: value.display_name,
            bio: value.bio,
            avatar: value.avatar,
            timestamp: value.created_at,
        }
    }
}

#[derive(Serialize)]
pub struct ChatMemberSchema {
    pub user_id: String,
    pub username: String,
    pub global_name: String,
    pub avatar: Option<String>,
    pub is_leave: bool,
    pub permissions: Option<String>,
}

#[derive(Serialize)]
pub struct ChatSchema {
    pub id: String,
    pub owner_id: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub image: Option<String>,
    pub chat_type: ChatType,
    pub last_message_id: Option<String>,
    pub permissions: String,
    pub timestamp: DateTime<Utc>,
    pub members: Vec<ChatMemberSchema>,
}

impl From<Chat> for ChatSchema {
    fn from(value: Chat) -> Self {
        Self {
            id: value.id.to_string(),
            owner_id: value.owner_id.map(|id| id.to_string()),
            name: value.name,
            description: Some("".to_string()),
            image: value.image,
            chat_type: value.chat_type,
            last_message_id: value.last_message_id.map(|id| id.to_string()),
            permissions: value.permissions.to_string(),
            timestamp: value.timestamp,
            members: value
                .members
                .into_iter()
                .map(|m| ChatMemberSchema {
                    user_id: m.user_id.to_string(),
                    username: m.username,
                    global_name: m.global_name,
                    avatar: m.avatar,
                    is_leave: m.is_leave,
                    permissions: m.permissions.map(|p| p.to_string()),
                })
                .collect(),
        }
    }
}

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

impl From<AddOrEditMessageCommandResult> for MessageSchema {
    fn from(result: AddOrEditMessageCommandResult) -> Self {
        let message = result.message;
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
            attachments: result.attachments.into_iter().map(|a| a.into()).collect(),
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
