use crate::assert_err;
use crate::{
    application::RequestHandler,
    application::messages::{
        CreateCloudAttachmentsCommand, CreateCloudAttachmentsCommandHandler, UploadAttachmentDto,
    },
    domain::chats::{Chat, ChatMember, ChatPermissions, CreateGroupChatRequest},
    error::Error,
    tests::common::TestContext,
};

#[tokio::test]
async fn test_create_cloud_attachments_success() {
    let ctx = TestContext::new("test_create_cloud_attachments_success").await;
    let handler = CreateCloudAttachmentsCommandHandler::new(&ctx.app_state);

    let owner = ctx.create_test_user("owner", "owner@test.com").await;
    let chat = Chat::create_group_dm(CreateGroupChatRequest {
        id: ctx.app_state.id_gen.gen_id().await,
        owner_id: owner.id,
        name: "Test Group".to_string(),
        permissions: Some(ChatPermissions::all()),
        members: vec![ChatMember::from(owner.clone())],
    });

    ctx.app_state
        .chat_repotisory
        .save(chat.clone())
        .await
        .expect("Failed to save chat");

    let cmd = CreateCloudAttachmentsCommand {
        current_user_id: owner.id,
        channel_id: chat.id,
        files: vec![
            UploadAttachmentDto {
                id: Some(1),
                filename: "test.txt".to_string(),
                filesize: 1024,
            },
            UploadAttachmentDto {
                id: Some(2),
                filename: "image.png".to_string(),
                filesize: 2048,
            },
        ],
    };

    let response = handler.handle(cmd).await.expect("Should succeed");

    assert_eq!(response.results.len(), 2);
    assert_eq!(response.errors.len(), 0);
    assert_eq!(response.results[0].id, Some(1));
    assert_eq!(response.results[1].id, Some(2));
    assert!(!response.results[0].upload_filename.is_empty());
    assert!(!response.results[0].upload_url.is_empty());
    assert!(
        response.results[0]
            .upload_filename
            .starts_with("attachments/")
    );
}

#[tokio::test]
async fn test_create_cloud_attachments_not_member() {
    let ctx = TestContext::new("test_create_cloud_attachments_not_member").await;
    let handler = CreateCloudAttachmentsCommandHandler::new(&ctx.app_state);

    let owner = ctx.create_test_user("owner", "owner@test.com").await;
    let non_member = ctx
        .create_test_user("nonmember", "nonmember@test.com")
        .await;

    let chat = Chat::create_group_dm(CreateGroupChatRequest {
        id: ctx.app_state.id_gen.gen_id().await,
        owner_id: owner.id,
        name: "Test Group".to_string(),
        permissions: Some(ChatPermissions::all()),
        members: vec![ChatMember::from(owner.clone())],
    });

    ctx.app_state
        .chat_repotisory
        .save(chat.clone())
        .await
        .expect("Failed to save chat");

    let cmd = CreateCloudAttachmentsCommand {
        current_user_id: non_member.id,
        channel_id: chat.id,
        files: vec![UploadAttachmentDto {
            id: Some(1),
            filename: "test.txt".to_string(),
            filesize: 1024,
        }],
    };

    let err = handler
        .handle(cmd)
        .await
        .expect_err("Should fail - not member");

    assert_err!(err, Error::UserNotMember { .. });
}

#[tokio::test]
async fn test_create_cloud_attachments_no_permission() {
    let ctx = TestContext::new("test_create_cloud_attachments_no_permission").await;
    let handler = CreateCloudAttachmentsCommandHandler::new(&ctx.app_state);

    let owner = ctx.create_test_user("owner", "owner@test.com").await;
    let member = ctx.create_test_user("member", "member@test.com").await;

    let limited_perms = ChatPermissions::SEND_MESSAGES;
    let chat = Chat::create_group_dm(CreateGroupChatRequest {
        id: ctx.app_state.id_gen.gen_id().await,
        owner_id: owner.id,
        name: "Test Group".to_string(),
        permissions: Some(limited_perms),
        members: vec![
            ChatMember::from(owner.clone()),
            ChatMember::from(member.clone()),
        ],
    });

    ctx.app_state
        .chat_repotisory
        .save(chat.clone())
        .await
        .expect("Failed to save chat");

    let cmd = CreateCloudAttachmentsCommand {
        current_user_id: member.id,
        channel_id: chat.id,
        files: vec![UploadAttachmentDto {
            id: Some(1),
            filename: "test.txt".to_string(),
            filesize: 1024,
        }],
    };

    let err = handler
        .handle(cmd)
        .await
        .expect_err("Should fail - no permission");

    assert_err!(err, Error::InsufficientPermissions { .. });
}

#[tokio::test]
async fn test_create_cloud_attachments_file_too_small() {
    let ctx = TestContext::new("test_create_cloud_attachments_file_too_small").await;
    let handler = CreateCloudAttachmentsCommandHandler::new(&ctx.app_state);

    let owner = ctx.create_test_user("owner", "owner@test.com").await;
    let chat = Chat::create_group_dm(CreateGroupChatRequest {
        id: ctx.app_state.id_gen.gen_id().await,
        owner_id: owner.id,
        name: "Test Group".to_string(),
        permissions: Some(ChatPermissions::all()),
        members: vec![ChatMember::from(owner.clone())],
    });

    ctx.app_state
        .chat_repotisory
        .save(chat.clone())
        .await
        .expect("Failed to save chat");

    let cmd = CreateCloudAttachmentsCommand {
        current_user_id: owner.id,
        channel_id: chat.id,
        files: vec![UploadAttachmentDto {
            id: Some(1),
            filename: "test.txt".to_string(),
            filesize: 0,
        }],
    };

    let response = handler.handle(cmd).await.expect("Should succeed");

    assert_eq!(response.results.len(), 0);
    assert_eq!(response.errors.len(), 1);
    assert_eq!(response.errors[0].id, Some(1));
    assert!(response.errors[0].errors[0].contains("File size"));
}

#[tokio::test]
async fn test_create_cloud_attachments_file_too_large() {
    let ctx = TestContext::new("test_create_cloud_attachments_file_too_large").await;
    let handler = CreateCloudAttachmentsCommandHandler::new(&ctx.app_state);

    let owner = ctx.create_test_user("owner", "owner@test.com").await;
    let chat = Chat::create_group_dm(CreateGroupChatRequest {
        id: ctx.app_state.id_gen.gen_id().await,
        owner_id: owner.id,
        name: "Test Group".to_string(),
        permissions: Some(ChatPermissions::all()),
        members: vec![ChatMember::from(owner.clone())],
    });

    ctx.app_state
        .chat_repotisory
        .save(chat.clone())
        .await
        .expect("Failed to save chat");

    let max_size = 10 * 1024 * 1024 + 1;
    let cmd = CreateCloudAttachmentsCommand {
        current_user_id: owner.id,
        channel_id: chat.id,
        files: vec![UploadAttachmentDto {
            id: Some(1),
            filename: "test.txt".to_string(),
            filesize: max_size,
        }],
    };

    let response = handler.handle(cmd).await.expect("Should succeed");

    assert_eq!(response.results.len(), 0);
    assert_eq!(response.errors.len(), 1);
    assert_eq!(response.errors[0].id, Some(1));
    assert!(response.errors[0].errors[0].contains("File size"));
}

#[tokio::test]
async fn test_create_cloud_attachments_mixed_valid_invalid() {
    let ctx = TestContext::new("test_create_cloud_attachments_mixed").await;
    let handler = CreateCloudAttachmentsCommandHandler::new(&ctx.app_state);

    let owner = ctx.create_test_user("owner", "owner@test.com").await;
    let chat = Chat::create_group_dm(CreateGroupChatRequest {
        id: ctx.app_state.id_gen.gen_id().await,
        owner_id: owner.id,
        name: "Test Group".to_string(),
        permissions: Some(ChatPermissions::all()),
        members: vec![ChatMember::from(owner.clone())],
    });

    ctx.app_state
        .chat_repotisory
        .save(chat.clone())
        .await
        .expect("Failed to save chat");

    let cmd = CreateCloudAttachmentsCommand {
        current_user_id: owner.id,
        channel_id: chat.id,
        files: vec![
            UploadAttachmentDto {
                id: Some(1),
                filename: "valid.txt".to_string(),
                filesize: 1024,
            },
            UploadAttachmentDto {
                id: Some(2),
                filename: "too_large.txt".to_string(),
                filesize: 20 * 1024 * 1024,
            },
        ],
    };

    let response = handler.handle(cmd).await.expect("Should succeed");

    assert_eq!(response.results.len(), 1);
    assert_eq!(response.errors.len(), 1);
    assert_eq!(response.results[0].id, Some(1));
    assert_eq!(response.errors[0].id, Some(2));
}
