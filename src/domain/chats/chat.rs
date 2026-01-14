use crate::domain::{auth::User, chats::chat_member::ChatMember};
use bitflags::bitflags;

pub enum ChatType {
    DM,
    GroupDM,
}

bitflags! {
    pub struct ChatPermissions: i64 {
        const SEND_MESSAGES = 1 << 0;
        const ADD_MEMBERS = 1 << 1;
        const PIN_MESSAGES = 1 << 2;
        const SEND_VIDEO_MESSAGES = 1 << 3;
        const SEND_VOICE_MESSAGES = 1 << 4;
        const SEND_FILES = 1 << 5;
        const CREATE_POLLS = 1 << 6;
        const CHANGE_GROUP_INFO = 1 << 7;

        const DELETE_MESSAGES = 1 << 8;
        const MANAGE_MEMBERS = 1 << 9;
        const MANAGE_INVITE_LINKS = 1 << 10;
        const ADD_ADMINS = 1 << 11;

        const DM_CHAT = Self::SEND_MESSAGES.bits() | Self::SEND_VIDEO_MESSAGES.bits()
            | Self::SEND_VOICE_MESSAGES.bits() | Self::DELETE_MESSAGES.bits()
            | Self::SEND_FILES.bits() | Self::PIN_MESSAGES.bits();

        const DEFAULT_DM_GROUP_MEMBER = Self::SEND_MESSAGES.bits() | Self::SEND_VIDEO_MESSAGES.bits()
            | Self::SEND_VOICE_MESSAGES.bits() | Self::ADD_MEMBERS.bits()
            | Self::SEND_FILES.bits() | Self::PIN_MESSAGES.bits()
            | Self::CREATE_POLLS.bits() | Self::CHANGE_GROUP_INFO.bits();
    }
}

pub struct Chat {
    pub id: i64,
    pub owner_id: Option<i64>,
    pub name: Option<String>,
    pub image: Option<String>,
    pub chat_type: ChatType,
    pub last_message_id: Option<i64>,
    pub permissions: ChatPermissions,

    members: Vec<super::chat_member::ChatMember>,
}

impl Chat {
    pub fn create_dm(id: i64, user_1: User, user_2: User) -> Self {
        Self {
            id: id,
            owner_id: None,
            name: None,
            image: None,
            chat_type: ChatType::DM,
            last_message_id: None,
            permissions: ChatPermissions::DM_CHAT,
            members: Vec::from([ChatMember::from(user_1), ChatMember::from(user_2)]),
        }
    }

    pub fn create_group_dm(id: i64, owner_id: i64, name: String, members: Vec<ChatMember>) -> Self {
        Self {
            id: id,
            owner_id: Some(owner_id),
            name: Some(name),
            image: None,
            chat_type: ChatType::GroupDM,
            last_message_id: None,
            permissions: ChatPermissions::DEFAULT_DM_GROUP_MEMBER,
            members: members,
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
    use crate::domain::auth::User;

    fn user(id: i64) -> User {
        User::create(
            id,
            format!("user{}", id),
            "Some password hash".to_ascii_lowercase(),
            format!("User {}", id),
            "test@test.com".to_string(),
        )
    }

    #[test]
    fn owner_has_any_permission() {
        let owner_id = 1;

        let chat = Chat {
            id: 1,
            owner_id: Some(owner_id),
            name: Some("test".into()),
            image: None,
            chat_type: ChatType::GroupDM,
            last_message_id: None,
            permissions: ChatPermissions::empty(),
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
            chat_type: ChatType::GroupDM,
            last_message_id: None,
            permissions: ChatPermissions::SEND_MESSAGES | ChatPermissions::SEND_FILES,
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
            chat_type: ChatType::GroupDM,
            last_message_id: None,
            permissions: ChatPermissions::SEND_MESSAGES | ChatPermissions::SEND_FILES,
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
            chat_type: ChatType::GroupDM,
            last_message_id: None,
            permissions: ChatPermissions::SEND_MESSAGES,
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
            chat_type: ChatType::GroupDM,
            last_message_id: None,
            permissions: ChatPermissions::SEND_MESSAGES,
            members: vec![member],
        };

        assert!(!chat.has_permission(user_id, ChatPermissions::SEND_MESSAGES));
    }

    #[test]
    fn dm_chat_default_permissions_work() {
        let user1 = user(1);
        let user2 = user(2);

        let chat = Chat::create_dm(1, user1, user2);

        assert!(chat.has_permission(1, ChatPermissions::SEND_MESSAGES));
        assert!(chat.has_permission(2, ChatPermissions::SEND_FILES));
        assert!(!chat.has_permission(1, ChatPermissions::ADD_MEMBERS));
    }
}
