use crate::{
    application::{
        RequestHandler,
        chats::{GetDMChannelCommand, GetDMChannelCommandHandler},
    },
    assert_err,
    domain::chats::{Chat, ChatMember, ChatType},
    error::Error,
    tests::common::TestContext,
};

#[tokio::test]
async fn test_get_dm_channel_existing() {
    let ctx = TestContext::new("test_get_dm_channel_existing").await;

    let user1 = ctx.create_test_user("user1", "user1@test.com").await;
    let user2 = ctx.create_test_user("user2", "user2@test.com").await;
    let _ = ctx.get_or_create_dm_chat(user1.id, user2.id).await;

    let handler = GetDMChannelCommandHandler::new(&ctx.app_state);
    let cmd = GetDMChannelCommand {
        current_user_id: user1.id,
        user_id: user2.id,
    };

    let result = handler.handle(cmd).await.expect("Should succeed");
    assert_eq!(result.chat_type, ChatType::Dm);
}

#[tokio::test]
async fn test_get_dm_channel_creates_new() {
    let ctx = TestContext::new("test_get_dm_channel_creates_new").await;

    let user1 = ctx.create_test_user("user1", "user1@test.com").await;
    let user2 = ctx.create_test_user("user2", "user2@test.com").await;

    let handler = GetDMChannelCommandHandler::new(&ctx.app_state);
    let cmd = GetDMChannelCommand {
        current_user_id: user1.id,
        user_id: user2.id,
    };

    let result = handler.handle(cmd).await.expect("Should succeed");
    assert_eq!(result.chat_type, ChatType::Dm);
    assert_eq!(result.members.len(), 2);
}

#[tokio::test]
async fn test_get_dm_channel_user_not_found() {
    let ctx = TestContext::new("test_get_dm_channel_user_not_found").await;

    let user1 = ctx.create_test_user("user1", "user1@test.com").await;

    let handler = GetDMChannelCommandHandler::new(&ctx.app_state);
    let cmd = GetDMChannelCommand {
        current_user_id: user1.id,
        user_id: 99999,
    };

    let err = handler
        .handle(cmd)
        .await
        .expect_err("Should fail - user not found");

    assert_err!(err, Error::UserNotFound(_));
}

#[tokio::test]
async fn test_get_dm_channel_same_user() {
    let ctx = TestContext::new("test_get_dm_channel_same_user").await;

    let user = ctx.create_test_user("testuser", "user@test.com").await;

    let handler = GetDMChannelCommandHandler::new(&ctx.app_state);
    let cmd = GetDMChannelCommand {
        current_user_id: user.id,
        user_id: user.id,
    };

    let result = handler.handle(cmd).await.expect("Should succeed");
    assert_eq!(result.chat_type, ChatType::Dm);
}
