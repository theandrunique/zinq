use tokio::sync::broadcast;

use crate::domain::{
    auth::User,
    chats::{Chat, ChatMember},
    messages::Message,
};

#[derive(Clone, Debug)]
pub enum DomainEvent {
    UserCreate {
        user: User,
    },
    ChatCreate {
        chat: Chat,
    },
    ChatMemberAdded {
        chat: Chat,
        member: ChatMember,
        initiator_id: i64,
    },
    ChatMemberRemoved {
        chat: Chat,
        member: ChatMember,
        initiator_id: i64,
    },
    MessageCreated {
        chat: Chat,
        message: Message,
        member: ChatMember,
    },
    MessageUpdated {
        chat: Chat,
        message: Message,
        member: ChatMember,
    },
}

pub struct EventBus {
    sender: broadcast::Sender<DomainEvent>,
}

impl EventBus {
    pub fn new() -> Self {
        // may be we need to use unbounded_channel instead of broadcast::channel
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
