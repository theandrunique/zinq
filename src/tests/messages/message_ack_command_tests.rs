use chrono::{Duration, TimeZone, Utc};

use crate::{
    application::{
        RequestHandler,
        messages::{MessageAckCommand, MessageAckCommandHandler, MessageAckInput},
    },
    assert_err,
    domain::messages::{Message, MessageType},
    error::Error,
    tests::common::TestContext,
};

#[tokio::test]
async fn test_ack_auto_fill() {
    let ctx = TestContext::new("test_ack_auto_fill").await;
    let user1 = ctx.create_test_user("user1", "user1@test.com").await;
    let user2 = ctx.create_test_user("user2", "user2@test.com").await;
    let chat = ctx
        .create_group_chat(user1.id, "Test chat", vec![user2.id], None)
        .await;

    let message1 = ctx.create_message(chat.id, user1.id, "message 1").await;
    let message2 = ctx.create_message(chat.id, user2.id, "message 2").await;
    let message3 = ctx.create_message(chat.id, user1.id, "message 3").await;

    let handler = MessageAckCommandHandler::new(&ctx.app_state);
    handler
        .handle(MessageAckCommand {
            current_user_id: user1.id,
            chat_id: chat.id,
            last_read_message_id: message3.message.id,
            acks: vec![],
        })
        .await
        .expect("Ack should succeed");

    for message_id in [
        message1.message.id,
        message2.message.id,
        message3.message.id,
    ] {
        let acks = ctx
            .app_state
            .message_ack_repository
            .get_acks(chat.id, message_id)
            .await
            .expect("Should fetch acks");

        assert_eq!(acks.len(), 1, "Message {} should have 1 ack", message_id);
        assert_eq!(acks[0].user_id, user1.id);
    }
}

#[tokio::test]
async fn test_ack_explicit() {
    let ctx = TestContext::new("test_ack_explicit").await;

    let user1 = ctx.create_test_user("user1", "user1@test.com").await;
    let user2 = ctx.create_test_user("user2", "user2@test.com").await;
    let chat = ctx
        .create_group_chat(user1.id, "Test chat", vec![user2.id], None)
        .await;

    let m1 = ctx.create_message(chat.id, user1.id, "message 1").await;
    let m2 = ctx.create_message(chat.id, user2.id, "message 2").await;
    let m3 = ctx.create_message(chat.id, user1.id, "message 3").await;

    let t1 = Utc.with_ymd_and_hms(2026, 6, 20, 10, 0, 0).unwrap();
    let t2 = Utc.with_ymd_and_hms(2026, 6, 20, 11, 0, 0).unwrap();
    let t3 = Utc.with_ymd_and_hms(2026, 6, 20, 12, 0, 0).unwrap();

    let handler = MessageAckCommandHandler::new(&ctx.app_state);
    handler
        .handle(MessageAckCommand {
            current_user_id: user1.id,
            chat_id: chat.id,
            last_read_message_id: m3.message.id,
            acks: vec![
                MessageAckInput {
                    message_id: m1.message.id,
                    acked_at: t1,
                },
                MessageAckInput {
                    message_id: m2.message.id,
                    acked_at: t2,
                },
                MessageAckInput {
                    message_id: m3.message.id,
                    acked_at: t3,
                },
            ],
        })
        .await
        .expect("Ack should succeed");

    let acks = ctx
        .app_state
        .message_ack_repository
        .get_acks(chat.id, m1.message.id)
        .await
        .expect("Should fetch acks");

    assert_eq!(acks.len(), 1, "Message {} should have 1 ack", m1.message.id);
    assert_eq!(acks[0].user_id, user1.id);
    assert_eq!(acks[0].created_at, t1);

    let acks = ctx
        .app_state
        .message_ack_repository
        .get_acks(chat.id, m2.message.id)
        .await
        .expect("Should fetch acks");

    assert_eq!(acks.len(), 1, "Message {} should have 1 ack", m2.message.id);
    assert_eq!(acks[0].user_id, user1.id);
    assert_eq!(acks[0].created_at, t2);

    let acks = ctx
        .app_state
        .message_ack_repository
        .get_acks(chat.id, m3.message.id)
        .await
        .expect("Should fetch acks");

    assert_eq!(acks.len(), 1, "Message {} should have 1 ack", m3.message.id);
    assert_eq!(acks[0].user_id, user1.id);
    assert_eq!(acks[0].created_at, t3);
}

#[tokio::test]
async fn test_ack_partial_explicit() {
    let ctx = TestContext::new("test_ack_partial_explicit").await;

    let user1 = ctx.create_test_user("user1", "user1@test.com").await;
    let user2 = ctx.create_test_user("user2", "user2@test.com").await;
    let chat = ctx
        .create_group_chat(user1.id, "Test chat", vec![user2.id], None)
        .await;

    let m1 = ctx.create_message(chat.id, user1.id, "message 1").await;
    let m2 = ctx.create_message(chat.id, user2.id, "message 2").await;
    let m3 = ctx.create_message(chat.id, user1.id, "message 3").await;

    let t1 = Utc.with_ymd_and_hms(2026, 6, 20, 10, 0, 0).unwrap();
    let t3 = Utc.with_ymd_and_hms(2026, 6, 20, 12, 0, 0).unwrap();

    let handler = MessageAckCommandHandler::new(&ctx.app_state);
    handler
        .handle(MessageAckCommand {
            current_user_id: user1.id,
            chat_id: chat.id,
            last_read_message_id: m3.message.id,
            acks: vec![
                MessageAckInput {
                    message_id: m1.message.id,
                    acked_at: t1,
                },
                MessageAckInput {
                    message_id: m3.message.id,
                    acked_at: t3,
                },
            ],
        })
        .await
        .expect("Ack should succeed");

    let acks = ctx
        .app_state
        .message_ack_repository
        .get_acks(chat.id, m1.message.id)
        .await
        .expect("Should fetch acks");

    assert_eq!(acks.len(), 1, "Message {} should have 1 ack", m1.message.id);
    assert_eq!(acks[0].user_id, user1.id);
    assert_eq!(acks[0].created_at, t1);

    let acks = ctx
        .app_state
        .message_ack_repository
        .get_acks(chat.id, m2.message.id)
        .await
        .expect("Should fetch acks");

    assert_eq!(acks.len(), 1, "Message {} should have 1 ack", m2.message.id);
    assert_eq!(acks[0].user_id, user1.id);
    assert_eq!(acks[0].created_at, t3);

    let acks = ctx
        .app_state
        .message_ack_repository
        .get_acks(chat.id, m3.message.id)
        .await
        .expect("Should fetch acks");

    assert_eq!(acks.len(), 1, "Message {} should have 1 ack", m3.message.id);
    assert_eq!(acks[0].user_id, user1.id);
    assert_eq!(acks[0].created_at, t3);
}

#[tokio::test]
async fn test_ack_idempotent() {
    let ctx = TestContext::new("test_ack_idempotent").await;

    let user1 = ctx.create_test_user("user1", "user1@test.com").await;
    let user2 = ctx.create_test_user("user2", "user2@test.com").await;
    let chat = ctx
        .create_group_chat(user1.id, "Test chat", vec![user2.id], None)
        .await;
    let message = ctx.create_message(chat.id, user1.id, "message 1").await;

    let handler = MessageAckCommandHandler::new(&ctx.app_state);

    for _ in 0..2 {
        handler
            .handle(MessageAckCommand {
                current_user_id: user1.id,
                chat_id: chat.id,
                last_read_message_id: message.message.id,
                acks: vec![],
            })
            .await
            .expect("Ack should succeed");
    }

    let acks = ctx
        .app_state
        .message_ack_repository
        .get_acks(chat.id, message.message.id)
        .await
        .expect("Should fetch acks");

    assert_eq!(acks.len(), 1, "Should have only 1 ack (idempotent)");
}

#[tokio::test]
async fn test_ack_updates_last_read_message_id() {
    let ctx = TestContext::new("test_ack_updates_last_read_message_id").await;

    let user1 = ctx.create_test_user("user1", "user1@test.com").await;
    let user2 = ctx.create_test_user("user2", "user2@test.com").await;
    let chat = ctx
        .create_group_chat(user1.id, "Test chat", vec![user2.id], None)
        .await;
    let message = ctx.create_message(chat.id, user1.id, "message 1").await;

    let handler = MessageAckCommandHandler::new(&ctx.app_state);
    handler
        .handle(MessageAckCommand {
            current_user_id: user1.id,
            chat_id: chat.id,
            last_read_message_id: message.message.id,
            acks: vec![],
        })
        .await
        .expect("Ack should succeed");

    let updated = ctx
        .app_state
        .chat_repository
        .get_by_id(chat.id)
        .await
        .expect("Should get chat")
        .expect("Chat should exists");

    let member = updated.get_member(user1.id).expect("Member should exist");

    assert_eq!(member.last_read_message_id, Some(message.message.id));
}

#[tokio::test]
async fn test_ack_not_member() {
    let ctx = TestContext::new("test_ack_not_member").await;

    let user1 = ctx.create_test_user("user1", "user1@test.com").await;
    let user2 = ctx.create_test_user("user2", "user2@test.com").await;
    let outsider = ctx.create_test_user("outsider", "outsider@test.com").await;
    let chat = ctx
        .create_group_chat(user1.id, "Test chat", vec![user2.id], None)
        .await;
    let message = ctx.create_message(chat.id, user1.id, "message 1").await;

    let handler = MessageAckCommandHandler::new(&ctx.app_state);
    let err = handler
        .handle(MessageAckCommand {
            current_user_id: outsider.id,
            chat_id: chat.id,
            last_read_message_id: message.message.id,
            acks: vec![],
        })
        .await
        .expect_err("Expected error");

    assert_err!(err, Error::UserNotMember { .. })
}

#[tokio::test]
async fn test_ack_chat_not_found() {
    let ctx = TestContext::new("test_ack_chat_not_found").await;

    let user = ctx.create_test_user("user1", "user@test.com").await;

    let handler = MessageAckCommandHandler::new(&ctx.app_state);
    let err = handler
        .handle(MessageAckCommand {
            current_user_id: user.id,
            chat_id: 99999,
            last_read_message_id: 100,
            acks: vec![],
        })
        .await
        .expect_err("Expected error");

    assert_err!(err, Error::ChatNotFound { .. })
}

#[tokio::test]
async fn test_ack_skips_old_messages() {
    let ctx = TestContext::new("test_ack_skips_old_messages").await;

    let user1 = ctx.create_test_user("user1", "user1@test.com").await;
    let user2 = ctx.create_test_user("user2", "user2@test.com").await;
    let chat = ctx
        .create_group_chat(user1.id, "Test chat", vec![user2.id], None)
        .await;

    let old_id = ((Utc::now() - Duration::days(10)) - ctx.app_state.id_gen.get_epoch())
        .num_milliseconds()
        << 22;

    let old_message = Message {
        id: old_id,
        chat_id: chat.id,
        author_id: user1.id,
        content: "old message".to_string(),
        created_at: Utc::now() - Duration::days(10),
        edited_at: None,
        message_type: MessageType::Default,
    };

    ctx.app_state
        .message_repository
        .upsert(&old_message)
        .await
        .expect("Should insert old message");

    let recent_message = ctx
        .create_message(chat.id, user1.id, "recent message")
        .await;

    let handler = MessageAckCommandHandler::new(&ctx.app_state);
    handler
        .handle(MessageAckCommand {
            current_user_id: user2.id,
            chat_id: chat.id,
            last_read_message_id: recent_message.message.id,
            acks: vec![],
        })
        .await
        .expect("Ack should succeed");

    let old_acks = ctx
        .app_state
        .message_ack_repository
        .get_acks(chat.id, old_id)
        .await
        .expect("Should fetch acks");
    assert_eq!(
        old_acks.len(),
        0,
        "Old message should not have ack (older than 7 days)"
    );

    let recent_acks = ctx
        .app_state
        .message_ack_repository
        .get_acks(chat.id, recent_message.message.id)
        .await
        .expect("Should fetch acks");
    assert_eq!(recent_acks.len(), 1, "Recent message should have ack");
}
