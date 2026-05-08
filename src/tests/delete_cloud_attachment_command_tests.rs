use crate::assert_err;
use crate::{
    application::RequestHandler,
    application::messages::{DeleteCloudAttachmentCommand, DeleteCloudAttachmentCommandHandler},
    domain::chats::{Chat, ChatMember, ChatPermissions, CreateGroupChatRequest},
    error::Error,
    tests::common::TestContext,
};

#[tokio::test]
async fn test_delete_cloud_attachment_not_found() {
    let ctx = TestContext::new("test_delete_cloud_attachment_not_found").await;
    let handler = DeleteCloudAttachmentCommandHandler::new(&ctx.app_state);

    let cmd = DeleteCloudAttachmentCommand {
        upload_filename: "attachments/123/456/nonexistent.txt".to_string(),
    };

    let err = handler
        .handle(cmd)
        .await
        .expect_err("Should fail - object not found");

    assert_err!(err, Error::AttachmentObjectNotFound { .. });
}

#[tokio::test]
async fn test_delete_cloud_attachment_invalid_filename() {
    let ctx = TestContext::new("test_delete_cloud_attachment_invalid_filename").await;
    let handler = DeleteCloudAttachmentCommandHandler::new(&ctx.app_state);

    let cmd = DeleteCloudAttachmentCommand {
        upload_filename: "invalid-format".to_string(),
    };

    let err = handler
        .handle(cmd)
        .await
        .expect_err("Should fail - invalid format");

    assert_err!(err, Error::AttachmentInvalidUploadFilename { .. });
}

#[tokio::test]
async fn test_delete_cloud_attachment_success_no_record() {
    let ctx = TestContext::new("test_delete_cloud_attachment_success").await;
    let delete_handler = DeleteCloudAttachmentCommandHandler::new(&ctx.app_state);

    let owner = ctx.create_test_user("owner", "owner@test.com").await;

    let chat_id = ctx.app_state.id_gen.gen_id().await;

    let chat = Chat::create_group_dm(CreateGroupChatRequest {
        id: chat_id,
        owner_id: owner.id,
        name: "Test Group".to_string(),
        permissions: Some(ChatPermissions::all()),
        members: vec![ChatMember::from(owner.clone())],
    });

    ctx.app_state
        .chat_repository
        .save(chat)
        .await
        .expect("Failed to save chat");

    let attachment_id = ctx.app_state.id_gen.gen_id().await;
    let upload_filename = format!("attachments/{}/{}/test.txt", chat_id, attachment_id);

    ctx.app_state
        .s3_service
        .put_object(&upload_filename, vec![1, 2, 3, 4], "text/plain")
        .await
        .expect("Failed to upload to S3");

    let cmd = DeleteCloudAttachmentCommand {
        upload_filename: upload_filename.clone(),
    };

    delete_handler.handle(cmd).await.expect("Should succeed");
}
