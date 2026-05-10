use crate::{
    application::{
        RequestHandler,
        chats::{CreateChatCommand, CreateChatCommandHandler},
        messages::{AddOrEditMessageCommand, AddOrEditMessageCommandHandler},
        messages::{GetMessagesQuery, GetMessagesQueryHandler},
    },
    assert_err,
    domain::messages::data::MessageRepository,
    error::Error,
    tests::common::TestContext,
};

#[tokio::test]
async fn test_get_messages_success() {
    let ctx = TestContext::new("test_get_messages_success").await;

    let current_user = ctx.create_test_user("currentuser", "owner@test.com").await;
    let chat = ctx.create_group_chat(current_user.id, "Test Group", vec![], None).await;
    let _msg1 = ctx.create_message(chat.id, current_user.id, "First").await;
    let msg2 = ctx.create_message(chat.id, current_user.id, "Second").await;
    let _msg3 = ctx.create_message(chat.id, current_user.id, "Third").await;

    let query_handler = GetMessagesQueryHandler::new(&ctx.app_state);

    let query_limit = GetMessagesQuery {
        current_user_id: current_user.id,
        chat_id: chat.id,
        before: None,
        limit: 2,
    };

    let result = query_handler
        .handle(query_limit)
        .await
        .expect("Should succeed with limit");
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].content, "Third");
    assert_eq!(result[1].content, "Second");

    let query_before = GetMessagesQuery {
        current_user_id: current_user.id,
        chat_id: chat.id,
        before: Some(msg2.message.id),
        limit: 2,
    };

    let result = query_handler
        .handle(query_before)
        .await
        .expect("Should succeed with before");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].content, "First");
}

#[tokio::test]
async fn test_get_messages_chat_not_found() {
    let ctx = TestContext::new("test_get_messages_not_found").await;

    let query_handler = GetMessagesQueryHandler::new(&ctx.app_state);
    let query = GetMessagesQuery {
        current_user_id: 1,
        chat_id: 99999,
        before: None,
        limit: 50,
    };

    let err = query_handler
        .handle(query)
        .await
        .expect_err("Should fail - chat not found");

    assert_err!(err, Error::ChatNotFound(_));
}

#[tokio::test]
async fn test_get_messages_not_member() {
    let ctx = TestContext::new("test_get_messages_not_member").await;

    let owner = ctx.create_test_user("owner", "owner@test.com").await;
    let stranger = ctx.create_test_user("stranger", "stranger@test.com").await;
    let chat = ctx.create_group_chat(owner.id, "Test Group", vec![], None).await;

    let query_handler = GetMessagesQueryHandler::new(&ctx.app_state);
    let query = GetMessagesQuery {
        current_user_id: stranger.id,
        chat_id: chat.id,
        before: None,
        limit: 50,
    };

    let err = query_handler
        .handle(query)
        .await
        .expect_err("Should fail - not member");

    assert_err!(err, Error::UserNotMember { .. });
}
