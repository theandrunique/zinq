use crate::{
    application::{
        RequestHandler,
        chats::{CreateChatCommand, CreateChatCommandHandler},
        chats::{GetAttachmentsQuery, GetAttachmentsQueryHandler},
        messages::{AddOrEditMessageCommand, AddOrEditMessageCommandHandler},
    },
    assert_err,
    domain::attachments::{Attachment, CreateAttachmentRequest},
    error::Error,
    tests::common::TestContext,
};

#[tokio::test]
async fn test_get_attachments_success() {
    let ctx = TestContext::new("test_get_attachments_success").await;

    let owner = ctx.create_test_user("owner", "owner@test.com").await;
    let chat = ctx
        .create_group_chat(owner.id, "Test Group", vec![], None)
        .await;
    let message = ctx.create_message(chat.id, owner.id, "testik").await;

    let attachment = Attachment::new(CreateAttachmentRequest {
        id: ctx.app_state.id_gen.gen_id().await,
        message_id: message.message.id,
        chat_id: chat.id,
        filename: "test.txt".to_string(),
        content_type: "text/plain".to_string(),
        size: 1024,
        storage_key: "test_key".to_string(),
    });

    ctx.app_state
        .attachment_repository
        .save(&attachment)
        .await
        .expect("Failed to save attachment");

    let query_handler = GetAttachmentsQueryHandler::new(&ctx.app_state);
    let query = GetAttachmentsQuery {
        current_user_id: owner.id,
        chat_id: chat.id,
        before: i64::MAX,
        limit: 50,
    };

    let result = query_handler.handle(query).await.expect("Should succeed");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].filename, "test.txt");
}

#[tokio::test]
async fn test_get_attachments_chat_not_found() {
    let ctx = TestContext::new("test_get_attachments_not_found").await;

    let query_handler = GetAttachmentsQueryHandler::new(&ctx.app_state);
    let query = GetAttachmentsQuery {
        current_user_id: 1,
        chat_id: 99999,
        before: i64::MAX,
        limit: 50,
    };

    let err = query_handler
        .handle(query)
        .await
        .expect_err("Should fail - chat not found");

    assert_err!(err, Error::ChatNotFound(_));
}

#[tokio::test]
async fn test_get_attachments_not_member() {
    let ctx = TestContext::new("test_get_attachments_not_member").await;

    let owner = ctx.create_test_user("owner", "owner@test.com").await;
    let stranger = ctx.create_test_user("stranger", "stranger@test.com").await;
    let chat = ctx
        .create_group_chat(owner.id, "Test Group", vec![], None)
        .await;

    let query_handler = GetAttachmentsQueryHandler::new(&ctx.app_state);
    let query = GetAttachmentsQuery {
        current_user_id: stranger.id,
        chat_id: chat.id,
        before: i64::MAX,
        limit: 50,
    };

    let err = query_handler
        .handle(query)
        .await
        .expect_err("Should fail - not member");

    assert_err!(err, Error::UserNotMember { .. });
}
