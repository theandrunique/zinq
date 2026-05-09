use crate::domain::{auth::User, chats::ChatPermissions};

#[derive(Clone, Debug)]
pub struct ChatMember {
    pub user_id: i64,
    pub last_read_message_id: Option<i64>,
    pub username: String,
    pub global_name: String,
    pub avatar: Option<String>,
    pub is_leave: bool,
    pub permissions: Option<ChatPermissions>,
}

impl ChatMember {
    pub fn set_leave_status(&mut self, is_leave: bool) {
        self.is_leave = is_leave;
    }
}

impl From<User> for ChatMember {
    fn from(value: User) -> Self {
        Self {
            user_id: value.id,
            last_read_message_id: None,
            username: value.username,
            global_name: value.display_name,
            avatar: value.avatar,
            is_leave: false,
            permissions: None,
        }
    }
}
