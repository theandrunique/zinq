use tokio::sync::broadcast;

use crate::domain::{
    auth::User,
    chats::{Chat, ChatMember},
};

#[derive(Clone, Debug)]
pub enum DomainEvent {
    UserCreate { user: User },
    ChatCreate { chat: Chat },
    ChatMemberAdded { chat: Chat, member: ChatMember },
    ChatMemberRemoved { chat_id: i64, user_id: i64 },
}

pub struct EventBus {
    sender: broadcast::Sender<DomainEvent>,
}

impl EventBus {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(2048);
        Self { sender }
    }

    pub fn publish(&self, event: DomainEvent) {
        let _ = self.sender.send(event);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<DomainEvent> {
        self.sender.subscribe()
    }
}
