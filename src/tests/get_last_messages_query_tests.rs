use tokio::sync::futures::OwnedNotified;

use crate::{
    application::{
        RequestHandler,
        chats::{CreateChatCommand, CreateChatCommandHandler},
        messages::{AddOrEditMessageCommand, AddOrEditMessageCommandHandler},
        messages::{GetLastMessagesQuery, GetLastMessagesQueryHandler},
    },
    tests::common::TestContext,
};

#[tokio::test]
async fn test_get_last_messages_empty() {
    let ctx = TestContext::new("test_get_last_messages_empty").await;

    let query_handler = GetLastMessagesQueryHandler::new(&ctx.app_state);
    let query = GetLastMessagesQuery {
        current_user_id: 1,
        chat_ids: vec![],
    };

    let result = query_handler.handle(query).await.expect("Should succeed");
    assert!(result.is_empty());
}

#[tokio::test]
async fn test_get_last_messages_returns_messages() {
    let ctx = TestContext::new("test_get_last_messages_returns").await;

    let current_user = ctx.create_test_user("currentuser", "owner@test.com").await;
    let chat = ctx
        .create_group_chat(current_user.id, "Test Group", vec![], None)
        .await;
    let _message = ctx.create_message(chat.id, current_user.id, "Hello").await;

    let query_handler = GetLastMessagesQueryHandler::new(&ctx.app_state);
    let query = GetLastMessagesQuery {
        current_user_id: current_user.id,
        chat_ids: vec![chat.id],
    };

    let result = query_handler.handle(query).await.expect("Should succeed");
    assert_eq!(result.len(), 1);
}
