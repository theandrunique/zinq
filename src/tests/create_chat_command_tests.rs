use crate::{
    application::{
        RequestHandler,
        chats::{CreateChatCommand, CreateChatCommandHandler},
    },
    assert_err,
    error::Error,
    tests::common::TestContext,
};

fn valid_command(current_user_id: i64, other_user_ids: Vec<i64>) -> CreateChatCommand {
    CreateChatCommand {
        current_user_id,
        name: "Test Chat".to_string(),
        members: other_user_ids,
    }
}

#[tokio::test]
async fn test_create_chat_command_success() {
    let ctx = TestContext::new("test_create_chat_success").await;
    let handler = CreateChatCommandHandler::new(&ctx.app_state);

    let current_user = ctx
        .create_test_user("currentuser", "current@test.com")
        .await;
    let other_user = ctx.create_test_user("otheruser", "other@test.com").await;

    let cmd = valid_command(current_user.id, vec![other_user.id]);

    let chat = handler
        .handle(cmd)
        .await
        .unwrap_or_else(|e| panic!("Create chat failed: {:?}", e));

    assert_eq!(chat.name, Some("Test Chat".to_string()));
    assert!(chat.has_member(current_user.id));
    assert!(chat.has_member(other_user.id));
}

#[tokio::test]
async fn test_create_chat_duplicate_members() {
    let ctx = TestContext::new("test_create_chat_duplicate_members").await;
    let handler = CreateChatCommandHandler::new(&ctx.app_state);

    let current_user = ctx
        .create_test_user("currentuser", "current@test.com")
        .await;
    let other_user = ctx.create_test_user("otheruser", "other@test.com").await;

    let cmd = valid_command(current_user.id, vec![other_user.id, other_user.id]);

    let err = handler
        .handle(cmd)
        .await
        .expect_err("Expected validation error for duplicate members");

    assert_err!(err, Error::InvalidRequestBody(_));
}

#[tokio::test]
async fn test_create_chat_missing_users() {
    let ctx = TestContext::new("test_create_chat_missing_users").await;
    let handler = CreateChatCommandHandler::new(&ctx.app_state);

    let current_user = ctx
        .create_test_user("currentuser", "current@test.com")
        .await;

    let nonexistent_id = 99999i64;
    let cmd = CreateChatCommand {
        current_user_id: current_user.id,
        name: "Test Chat".to_string(),
        members: vec![nonexistent_id],
    };

    let err = handler
        .handle(cmd)
        .await
        .expect_err("Expected UsersNotFound error");

    assert_err!(err, Error::UsersNotFound(_));
}

#[tokio::test]
async fn test_create_chat_saved_in_repository() {
    let ctx = TestContext::new("test_create_chat_saved").await;
    let handler = CreateChatCommandHandler::new(&ctx.app_state);

    let current_user = ctx
        .create_test_user("currentuser", "current@test.com")
        .await;
    let other_user = ctx.create_test_user("otheruser", "other@test.com").await;

    let cmd = valid_command(current_user.id, vec![other_user.id]);

    let chat = handler.handle(cmd).await.unwrap();

    let stored_chat = ctx
        .app_state
        .chat_repotisory
        .get_by_id(chat.id)
        .await
        .unwrap_or_else(|e| panic!("Failed to get chat by id: {:?}", e))
        .expect("Chat should exist in repository");
    assert_eq!(stored_chat.id, chat.id);
}

#[tokio::test]
async fn test_create_chat_current_user_not_in_members() {
    let ctx = TestContext::new("test_create_chat_auto_add").await;
    let handler = CreateChatCommandHandler::new(&ctx.app_state);

    let current_user = ctx
        .create_test_user("currentuser", "current@test.com")
        .await;
    let other_user = ctx.create_test_user("otheruser", "other@test.com").await;

    let cmd = CreateChatCommand {
        current_user_id: current_user.id,
        name: "Test Chat".to_string(),
        members: vec![other_user.id],
    };

    let chat = handler.handle(cmd).await.unwrap();

    assert!(chat.has_member(current_user.id));
    assert!(chat.has_member(other_user.id));
}

#[tokio::test]
async fn test_create_chat_empty_members() {
    let ctx = TestContext::new("test_create_chat_empty_members").await;
    let handler = CreateChatCommandHandler::new(&ctx.app_state);

    let current_user = ctx
        .create_test_user("currentuser", "current@test.com")
        .await;

    let cmd = CreateChatCommand {
        current_user_id: current_user.id,
        name: "Test Chat".to_string(),
        members: vec![],
    };

    let chat = handler
        .handle(cmd)
        .await
        .unwrap_or_else(|e| panic!("Empty members should create chat with self: {:?}", e));

    assert!(chat.has_member(current_user.id));
}

#[tokio::test]
async fn test_create_chat_only_current_user() {
    let ctx = TestContext::new("test_create_chat_only_self").await;
    let handler = CreateChatCommandHandler::new(&ctx.app_state);

    let current_user = ctx
        .create_test_user("currentuser", "current@test.com")
        .await;

    let cmd = CreateChatCommand {
        current_user_id: current_user.id,
        name: "Solo Chat".to_string(),
        members: vec![],
    };

    let chat = handler
        .handle(cmd)
        .await
        .unwrap_or_else(|e| panic!("Create chat with only self failed: {:?}", e));

    assert!(chat.has_member(current_user.id));
}

#[tokio::test]
async fn test_create_chat_current_user_in_members_explicitly() {
    let ctx = TestContext::new("test_create_chat_explicit_self").await;
    let handler = CreateChatCommandHandler::new(&ctx.app_state);

    let current_user = ctx
        .create_test_user("currentuser", "current@test.com")
        .await;
    let other_user = ctx.create_test_user("otheruser", "other@test.com").await;

    let cmd = CreateChatCommand {
        current_user_id: current_user.id,
        name: "Test Chat".to_string(),
        members: vec![current_user.id, other_user.id],
    };

    let chat = handler.handle(cmd).await.unwrap();

    assert!(chat.has_member(current_user.id));
    assert!(chat.has_member(other_user.id));
}

#[tokio::test]
async fn test_create_chat_publishes_event() {
    let ctx = TestContext::new("test_create_chat_event").await;
    let handler = CreateChatCommandHandler::new(&ctx.app_state);

    let current_user = ctx
        .create_test_user("currentuser", "current@test.com")
        .await;
    let other_user = ctx.create_test_user("otheruser", "other@test.com").await;

    let mut receiver = ctx.app_state.event_bus.subscribe();

    let cmd = valid_command(current_user.id, vec![other_user.id]);

    let _ = handler.handle(cmd).await.unwrap();

    let event = tokio::time::timeout(std::time::Duration::from_secs(1), receiver.recv())
        .await
        .expect("Should receive event")
        .expect("Event should be available");

    assert!(
        matches!(event, crate::domain::events::DomainEvent::ChatCreate { .. }),
        "Event should be ChatCreate"
    );
}

#[tokio::test]
async fn test_create_chat_with_empty_name() {
    let ctx = TestContext::new("test_create_chat_empty_name").await;
    let handler = CreateChatCommandHandler::new(&ctx.app_state);

    let current_user = ctx
        .create_test_user("currentuser", "current@test.com")
        .await;
    let other_user = ctx.create_test_user("otheruser", "other@test.com").await;

    let cmd = CreateChatCommand {
        current_user_id: current_user.id,
        name: "".to_string(),
        members: vec![other_user.id],
    };

    let chat = handler
        .handle(cmd)
        .await
        .unwrap_or_else(|e| panic!("Empty name should work: {:?}", e));

    assert_eq!(chat.name, Some("".to_string()));
}

#[tokio::test]
async fn test_create_chat_multiple_users() {
    let ctx = TestContext::new("test_create_chat_multiple_users").await;
    let handler = CreateChatCommandHandler::new(&ctx.app_state);

    let current_user = ctx.create_test_user("user1", "user1@test.com").await;
    let user2 = ctx.create_test_user("user2", "user2@test.com").await;
    let user3 = ctx.create_test_user("user3", "user3@test.com").await;

    let cmd = CreateChatCommand {
        current_user_id: current_user.id,
        name: "Group Chat".to_string(),
        members: vec![user2.id, user3.id],
    };

    let chat = handler.handle(cmd).await.unwrap();

    assert!(chat.has_member(current_user.id));
    assert!(chat.has_member(user2.id));
    assert!(chat.has_member(user3.id));
}
