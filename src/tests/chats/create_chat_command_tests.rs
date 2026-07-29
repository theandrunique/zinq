use crate::{
    application::{
        RequestHandler,
        chats::{CreateChatCommand, CreateChatCommandHandler},
    },
    assert_err,
    domain::events::DomainEvent,
    error::Error,
    tests::common::TestContext,
};

#[tokio::test]
async fn test_create_chat_command_success() {
    let ctx = TestContext::new("test_create_chat_command_success").await;

    let current_user = ctx
        .create_test_user("currentuser", "currentuser@test.com")
        .await;
    let member = ctx.create_test_user("member", "member@test.com").await;

    let handler = CreateChatCommandHandler::new(&ctx.app_state);
    let cmd = CreateChatCommand {
        current_user_id: current_user.id,
        name: "Test Chat".to_string(),
        members: vec![member.id],
        permissions: None,
    };

    let chat = handler
        .handle(cmd)
        .await
        .expect("Create chat should succeed");

    assert_eq!(chat.name, Some("Test Chat".to_string()));
    assert!(chat.has_member(current_user.id));
    assert!(chat.has_member(member.id));
}

#[tokio::test]
async fn test_create_chat_duplicate_members() {
    let ctx = TestContext::new("test_create_chat_duplicate_members").await;

    let current_user = ctx
        .create_test_user("currentuser", "current@test.com")
        .await;
    let member = ctx.create_test_user("member", "member@test.com").await;

    let handler = CreateChatCommandHandler::new(&ctx.app_state);
    let cmd = CreateChatCommand {
        current_user_id: current_user.id,
        name: "Test Chat".to_string(),
        members: vec![member.id, member.id],
        permissions: None,
    };

    let err = handler
        .handle(cmd)
        .await
        .expect_err("Expected validation error for duplicate members");

    assert_err!(err, Error::InvalidRequestBody(_));
}

#[tokio::test]
async fn test_create_chat_missing_users() {
    let ctx = TestContext::new("test_create_chat_missing_users").await;

    let current_user = ctx
        .create_test_user("currentuser", "current@test.com")
        .await;

    let handler = CreateChatCommandHandler::new(&ctx.app_state);
    let cmd = CreateChatCommand {
        current_user_id: current_user.id,
        name: "Test Chat".to_string(),
        members: vec![99999],
        permissions: None,
    };

    let err = handler
        .handle(cmd)
        .await
        .expect_err("Expected UsersNotFound error");

    assert_err!(err, Error::UsersNotFound(_));
}

#[tokio::test]
async fn test_create_chat_saved_in_repository() {
    let ctx = TestContext::new("test_create_chat_saved_in_repository").await;

    let current_user = ctx
        .create_test_user("currentuser", "currentuser@test.com")
        .await;
    let member = ctx.create_test_user("member", "member@test.com").await;

    let handler = CreateChatCommandHandler::new(&ctx.app_state);
    let cmd = CreateChatCommand {
        current_user_id: current_user.id,
        name: "Test Chat".to_string(),
        members: vec![member.id],
        permissions: None,
    };

    let chat = handler
        .handle(cmd)
        .await
        .expect("Create chat should succeed");

    let stored_chat = ctx
        .app_state
        .chat_repository
        .get_by_id(chat.id)
        .await
        .expect("Failed to get chat by id")
        .expect("Chat should exist in repository");

    assert_eq!(stored_chat.id, chat.id);
}

#[tokio::test]
async fn test_create_chat_current_user_not_in_members() {
    let ctx = TestContext::new("test_create_chat_current_user_not_in_members").await;

    let current_user = ctx
        .create_test_user("currentuser", "currentuser@test.com")
        .await;
    let other_user = ctx
        .create_test_user("otheruser", "otheruser@test.com")
        .await;

    let handler = CreateChatCommandHandler::new(&ctx.app_state);
    let cmd = CreateChatCommand {
        current_user_id: current_user.id,
        name: "Test Chat".to_string(),
        members: vec![other_user.id],
        permissions: None,
    };

    let chat = handler
        .handle(cmd)
        .await
        .expect("Create chat should succeed");

    assert!(chat.has_member(current_user.id));
    assert!(chat.has_member(other_user.id));
}

#[tokio::test]
async fn test_create_chat_empty_members() {
    let ctx = TestContext::new("test_create_chat_empty_members").await;

    let current_user = ctx
        .create_test_user("currentuser", "current@test.com")
        .await;

    let handler = CreateChatCommandHandler::new(&ctx.app_state);
    let cmd = CreateChatCommand {
        current_user_id: current_user.id,
        name: "Test Chat".to_string(),
        members: vec![],
        permissions: None,
    };

    let chat = handler
        .handle(cmd)
        .await
        .expect("Create chat should succeed");

    assert!(chat.has_member(current_user.id));
}

#[tokio::test]
async fn test_create_chat_only_current_user() {
    let ctx = TestContext::new("test_create_chat_only_current_user").await;

    let current_user = ctx
        .create_test_user("currentuser", "current@test.com")
        .await;

    let handler = CreateChatCommandHandler::new(&ctx.app_state);
    let cmd = CreateChatCommand {
        current_user_id: current_user.id,
        name: "Test Chat".to_string(),
        members: vec![current_user.id],
        permissions: None,
    };

    let chat = handler
        .handle(cmd)
        .await
        .expect("Create chat should succeed");

    assert!(chat.has_member(current_user.id));
}

#[tokio::test]
async fn test_create_chat_current_user_in_members() {
    let ctx = TestContext::new("test_create_chat_current_user_in_members").await;

    let current_user = ctx
        .create_test_user("currentuser", "currentuser@test.com")
        .await;
    let other_user = ctx
        .create_test_user("otheruser", "otheruser@test.com")
        .await;

    let handler = CreateChatCommandHandler::new(&ctx.app_state);
    let cmd = CreateChatCommand {
        current_user_id: current_user.id,
        name: "Test Chat".to_string(),
        members: vec![current_user.id, other_user.id],
        permissions: None,
    };

    let chat = handler
        .handle(cmd)
        .await
        .expect("Create chat should succeed");

    assert!(chat.has_member(current_user.id));
    assert!(chat.has_member(other_user.id));
}

#[tokio::test]
async fn test_create_chat_with_empty_name() {
    let ctx = TestContext::new("test_create_chat_with_empty_name").await;

    let current_user = ctx
        .create_test_user("currentuser", "currentuser@test.com")
        .await;
    let other_user = ctx
        .create_test_user("otheruser", "otheruser@test.com")
        .await;

    let handler = CreateChatCommandHandler::new(&ctx.app_state);
    let cmd = CreateChatCommand {
        current_user_id: current_user.id,
        name: "".to_string(),
        members: vec![other_user.id],
        permissions: None,
    };

    let chat = handler
        .handle(cmd)
        .await
        .expect("Create chat should succeed");

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
        name: "Test Group".to_string(),
        members: vec![user2.id, user3.id],
        permissions: None,
    };

    let chat = handler
        .handle(cmd)
        .await
        .expect("Create chat should succeed");

    assert!(chat.has_member(current_user.id));
    assert!(chat.has_member(user2.id));
    assert!(chat.has_member(user3.id));
}
