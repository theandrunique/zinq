use std::str::FromStr;

use crate::domain::{
    auth::User,
    chats::{chat_member::ChatMember, chat_permissions::ChatPermissions},
};
use bitflags::bitflags;
use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ChatType {
    Dm,
    GroupDm,
}

#[derive(Clone, Debug)]
pub struct Chat {
    pub id: i64,
    pub owner_id: Option<i64>,
    pub name: Option<String>,
    pub image: Option<String>,
    pub chat_type: ChatType,
    pub last_message_id: Option<i64>,
    pub permissions: ChatPermissions,
    pub timestamp: DateTime<Utc>,

    pub members: Vec<ChatMember>,
}

pub struct CreateGroupChatRequest {
    pub id: i64,
    pub owner_id: i64,
    pub name: String,
    pub members: Vec<ChatMember>,
    pub permissions: Option<ChatPermissions>,
}

impl Chat {
    pub fn create_dm(id: i64, members: Vec<ChatMember>) -> Self {
        Self {
            id: id,
            owner_id: None,
            name: None,
            image: None,
            chat_type: ChatType::Dm,
            last_message_id: None,
            permissions: ChatPermissions::DM_CHAT,
            timestamp: Utc::now(),
            members: members,
        }
    }

    pub fn create_group_dm(request: CreateGroupChatRequest) -> Self {
        Self {
            id: request.id,
            owner_id: Some(request.owner_id),
            name: Some(request.name),
            image: None,
            chat_type: ChatType::GroupDm,
            last_message_id: None,
            permissions: request
                .permissions
                .unwrap_or(ChatPermissions::DEFAULT_GROUP_DM_MEMBER),
            timestamp: Utc::now(),
            members: request.members,
        }
    }

    pub fn get_member(&self, user_id: i64) -> Option<&ChatMember> {
        match self
            .members
            .iter()
            .find(|m| m.user_id == user_id && !m.is_leave)
        {
            Some(m) => Some(m),
            None => return None,
        }
    }

    pub fn has_member(&self, user_id: i64) -> bool {
        self.get_member(user_id).is_some()
    }

    pub fn has_permission(&self, user_id: i64, permission: ChatPermissions) -> bool {
        if self.owner_id == Some(user_id) {
            return true;
        }

        let member = match self
            .members
            .iter()
            .find(|m| m.user_id == user_id && !m.is_leave)
        {
            Some(m) => m,
            None => return false,
        };

        if let Some(member_permissions) = member.permissions.as_ref() {
            return member_permissions.contains(permission);
        }

        self.permissions.contains(permission)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::auth::{User, UserCreateRequest};

    fn user(id: i64) -> User {
        User::create(UserCreateRequest {
            id: id,
            username: format!("user{}", id),
            password_hash: "kek".to_string(),
            display_name: format!("User {}", id),
            email: "test@test.com".to_string(),
        })
    }

    #[test]
    fn owner_has_any_permission() {
        let owner_id = 1;

        let chat = Chat {
            id: 1,
            owner_id: Some(owner_id),
            name: Some("test".into()),
            image: None,
            chat_type: ChatType::GroupDm,
            last_message_id: None,
            permissions: ChatPermissions::empty(),
            timestamp: Utc::now(),
            members: vec![],
        };

        assert!(chat.has_permission(owner_id, ChatPermissions::MANAGE_MEMBERS));
        assert!(chat.has_permission(owner_id, ChatPermissions::MANAGE_INVITE_LINKS));
        assert!(chat.has_permission(owner_id, ChatPermissions::DELETE_MESSAGES));
        assert!(chat.has_permission(owner_id, ChatPermissions::ADD_ADMINS));
    }

    #[test]
    fn member_uses_chat_permissions_when_no_override() {
        let user_id = 1;

        let member = ChatMember::from(user(user_id));

        let chat = Chat {
            id: 1,
            owner_id: None,
            name: None,
            image: None,
            chat_type: ChatType::GroupDm,
            last_message_id: None,
            permissions: ChatPermissions::SEND_MESSAGES | ChatPermissions::SEND_FILES,
            timestamp: Utc::now(),
            members: vec![member],
        };

        assert!(chat.has_permission(user_id, ChatPermissions::SEND_MESSAGES));
        assert!(chat.has_permission(user_id, ChatPermissions::SEND_FILES));
        assert!(!chat.has_permission(user_id, ChatPermissions::SEND_VIDEO_MESSAGES));
    }

    #[test]
    fn member_permissions_override_chat_permissions() {
        let user_id = 1;

        let mut member = ChatMember::from(user(user_id));
        member.permissions = Some(ChatPermissions::SEND_MESSAGES);

        let chat = Chat {
            id: 1,
            owner_id: None,
            name: None,
            image: None,
            chat_type: ChatType::GroupDm,
            last_message_id: None,
            permissions: ChatPermissions::SEND_MESSAGES | ChatPermissions::SEND_FILES,
            timestamp: Utc::now(),
            members: vec![member],
        };

        assert!(chat.has_permission(user_id, ChatPermissions::SEND_MESSAGES));
        assert!(!chat.has_permission(user_id, ChatPermissions::SEND_FILES));
    }

    #[test]
    fn non_member_has_no_permissions() {
        let chat = Chat {
            id: 1,
            owner_id: None,
            name: None,
            image: None,
            chat_type: ChatType::GroupDm,
            last_message_id: None,
            permissions: ChatPermissions::SEND_MESSAGES,
            timestamp: Utc::now(),
            members: vec![],
        };

        assert!(!chat.has_permission(42, ChatPermissions::SEND_MESSAGES));
    }

    #[test]
    fn left_member_has_no_permissions() {
        let user_id = 1;

        let mut member = ChatMember::from(user(user_id));
        member.is_leave = true;

        let chat = Chat {
            id: 1,
            owner_id: None,
            name: None,
            image: None,
            chat_type: ChatType::GroupDm,
            last_message_id: None,
            permissions: ChatPermissions::SEND_MESSAGES,
            timestamp: Utc::now(),
            members: vec![member],
        };

        assert!(!chat.has_permission(user_id, ChatPermissions::SEND_MESSAGES));
    }

    #[test]
    fn dm_chat_default_permissions_work() {
        let user1 = user(1);
        let user2 = user(2);

        let members = vec![ChatMember::from(user1), ChatMember::from(user2)];
        let chat = Chat::create_dm(1, members);

        assert!(chat.has_permission(1, ChatPermissions::SEND_MESSAGES));
        assert!(chat.has_permission(2, ChatPermissions::SEND_FILES));
        assert!(!chat.has_permission(1, ChatPermissions::ADD_MEMBERS));
    }
}
