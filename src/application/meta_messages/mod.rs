use anyhow::Context;
use tokio::sync::broadcast;

use crate::{
    domain::{
        chats::{Chat, ChatMember, ChatType},
        events::DomainEvent,
        messages::{CreateMetaMessageRequest, Message, MessageType},
    },
    state::AppState,
};

pub fn start_meta_message_worker(app_state: AppState) {
    tokio::spawn(meta_message_worker(
        app_state.clone(),
        app_state.event_bus.subscribe(),
    ));
}

async fn meta_message_worker(app_state: AppState, mut events: broadcast::Receiver<DomainEvent>) {
    while let Ok(event) = events.recv().await {
        match event {
            DomainEvent::UserCreate { .. } => {}
            DomainEvent::MessageCreated { .. } => {}
            DomainEvent::MessageUpdated { .. } => {}
            DomainEvent::ChatCreate { chat } => {
                if let Err(e) = handle_chat_create(&app_state, chat).await {
                    tracing::error!("Failed to handle ChatCreate: {:#}", e);
                }
            }
            DomainEvent::ChatMemberAdded {
                chat,
                member,
                initiator_id,
            } => {
                if let Err(e) = handle_chat_member_add(&app_state, chat, member, initiator_id).await
                {
                    tracing::error!("Failed to handle ChatMemberAdded: {:#}", e);
                }
            }
            DomainEvent::ChatMemberRemoved {
                chat,
                member,
                initiator_id,
            } => {
                if let Err(e) =
                    handle_chat_member_remove(&app_state, chat, member, initiator_id).await
                {
                    tracing::error!("Failed to handle ChatMemberRemoved: {:#}", e);
                }
            }
        }
    }
}

async fn handle_chat_create(app_state: &AppState, chat: Chat) -> anyhow::Result<()> {
    if chat.chat_type != ChatType::GroupDm {
        return Ok(());
    }

    let name = chat
        .name
        .as_ref()
        .with_context(|| format!("Chat name was None for {}", chat.id))?;

    let owner_id = chat
        .owner_id
        .with_context(|| format!("Chat owner_id was None for {}", chat.id))?;

    let member = chat
        .get_member(owner_id)
        .with_context(|| format!("Chat member not found for owner_id {}", owner_id))?;

    let chat_name_owned = name.clone();

    let meta_message = Message::new_meta(CreateMetaMessageRequest {
        id: app_state.id_gen.gen_id().await,
        chat_id: chat.id,
        author_id: owner_id,
        message_type: MessageType::ChatCreate {
            chat_name: chat_name_owned,
        },
    });

    app_state
        .message_repository
        .upsert(meta_message.clone())
        .await?;
    app_state.event_bus.publish(DomainEvent::MessageCreated {
        chat,
        message: meta_message,
        member,
    });

    Ok(())
}

async fn handle_chat_member_add(
    app_state: &AppState,
    chat: Chat,
    member: ChatMember,
    initiator_id: i64,
) -> anyhow::Result<()> {
    let initiator = chat
        .get_member(initiator_id)
        .with_context(|| format!("Chat member not found for owner_id {}", initiator_id))?;

    let meta_message = Message::new_meta(CreateMetaMessageRequest {
        id: app_state.id_gen.gen_id().await,
        chat_id: chat.id,
        author_id: initiator_id,
        message_type: MessageType::MemberAdd {
            user_id: member.user_id,
        },
    });

    app_state
        .message_repository
        .upsert(meta_message.clone())
        .await?;
    app_state.event_bus.publish(DomainEvent::MessageCreated {
        chat,
        message: meta_message,
        member: initiator,
    });

    Ok(())
}

async fn handle_chat_member_remove(
    app_state: &AppState,
    chat: Chat,
    member: ChatMember,
    initiator_id: i64,
) -> anyhow::Result<()> {
    let initiator = chat
        .get_member(initiator_id)
        .with_context(|| format!("Chat member not found for owner_id {}", initiator_id))?;

    let message_type = if member.user_id == initiator_id {
        MessageType::MemberLeave {
            user_id: initiator_id,
        }
    } else {
        MessageType::MemberRemove {
            user_id: member.user_id,
        }
    };

    let meta_message = Message::new_meta(CreateMetaMessageRequest {
        id: app_state.id_gen.gen_id().await,
        chat_id: chat.id,
        author_id: initiator_id,
        message_type: message_type,
    });

    app_state
        .message_repository
        .upsert(meta_message.clone())
        .await?;
    app_state.event_bus.publish(DomainEvent::MessageCreated {
        chat,
        message: meta_message,
        member: initiator,
    });

    Ok(())
}
