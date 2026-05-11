use crate::{
    application::{
        RequestHandler,
        messages::{
            AddOrEditMessageCommand, AddOrEditMessageCommandHandler, CreateCloudAttachmentsCommand,
            CreateCloudAttachmentsCommandHandler, MessageAttachmentInput, UploadAttachmentDto,
        },
    },
    assert_err,
    domain::{chats::ChatPermissions, events::DomainEvent, messages::MessageType},
    error::Error,
    tests::common::TestContext,
};

fn valid_command(current_user_id: i64, chat_id: i64) -> AddOrEditMessageCommand {
    AddOrEditMessageCommand {
        current_user_id,
        message_id: None,
        referenced_message_id: None,
        chat_id,
        content: "Hello world".to_string(),
        attachments: vec![],
    }
}

#[tokio::test]
async fn test_add_message_command_success() {
    let ctx = TestContext::new("test_add_message_success").await;

    let current_user = ctx
        .create_test_user("currentuser", "current@test.com")
        .await;
    let other_user = ctx.create_test_user("otheruser", "other@test.com").await;

    let chat = ctx
        .create_group_chat(current_user.id, "Test Chat", vec![other_user.id], None)
        .await;

    let cmd = AddOrEditMessageCommand {
        current_user_id: current_user.id,
        message_id: None,
        referenced_message_id: None,
        chat_id: chat.id,
        content: "Hello world".to_string(),
        attachments: vec![],
    };

    let handler = AddOrEditMessageCommandHandler::new(&ctx.app_state);

    let result = handler
        .handle(cmd)
        .await
        .unwrap_or_else(|e| panic!("Add message failed: {:?}", e));

    assert_eq!(result.message.content, "Hello world");
    assert_eq!(result.message.chat_id, chat.id);
    assert_eq!(result.message.author_id, current_user.id);
    assert!(matches!(result.message.message_type, MessageType::Default));
}

#[tokio::test]
async fn test_add_message_with_reply() {
    let ctx = TestContext::new("test_add_message_reply").await;

    let current_user = ctx
        .create_test_user("currentuser", "current@test.com")
        .await;
    let other_user = ctx.create_test_user("otheruser", "other@test.com").await;

    let chat = ctx
        .create_group_chat(current_user.id, "Test Chat", vec![other_user.id], None)
        .await;

    let first_message_cmd = AddOrEditMessageCommand {
        current_user_id: current_user.id,
        message_id: None,
        referenced_message_id: None,
        chat_id: chat.id,
        content: "First message".to_string(),
        attachments: vec![],
    };

    let handler = AddOrEditMessageCommandHandler::new(&ctx.app_state);

    let first_result = handler
        .handle(first_message_cmd)
        .await
        .unwrap_or_else(|e| panic!("Add first message failed: {:?}", e));

    let reply_cmd = AddOrEditMessageCommand {
        current_user_id: current_user.id,
        message_id: None,
        referenced_message_id: Some(first_result.message.id),
        chat_id: chat.id,
        content: "Reply message".to_string(),
        attachments: vec![],
    };

    let reply_result = handler
        .handle(reply_cmd)
        .await
        .unwrap_or_else(|e| panic!("Add reply failed: {:?}", e));

    assert_eq!(reply_result.message.content, "Reply message");
    assert!(matches!(
        reply_result.message.message_type,
        MessageType::Reply {
            referenced_message_id: _
        }
    ));
}

#[tokio::test]
async fn test_edit_message_success() {
    let ctx = TestContext::new("test_edit_message_success").await;

    let current_user = ctx
        .create_test_user("currentuser", "current@test.com")
        .await;
    let other_user = ctx.create_test_user("otheruser", "other@test.com").await;

    let chat = ctx
        .create_group_chat(current_user.id, "Test Chat", vec![other_user.id], None)
        .await;

    let handler = AddOrEditMessageCommandHandler::new(&ctx.app_state);

    let create_cmd = AddOrEditMessageCommand {
        current_user_id: current_user.id,
        message_id: None,
        referenced_message_id: None,
        chat_id: chat.id,
        content: "Original content".to_string(),
        attachments: vec![],
    };

    let create_result = handler
        .handle(create_cmd)
        .await
        .unwrap_or_else(|e| panic!("Create message failed: {:?}", e));

    let edit_cmd = AddOrEditMessageCommand {
        current_user_id: current_user.id,
        message_id: Some(create_result.message.id),
        referenced_message_id: None,
        chat_id: chat.id,
        content: "Edited content".to_string(),
        attachments: vec![],
    };

    let edit_result = handler
        .handle(edit_cmd)
        .await
        .unwrap_or_else(|e| panic!("Edit message failed: {:?}", e));

    assert_eq!(edit_result.message.content, "Edited content");
    assert!(edit_result.message.edited_at.is_some());
}

#[tokio::test]
async fn test_edit_message_not_owner() {
    let ctx = TestContext::new("test_edit_message_not_owner").await;

    let current_user = ctx
        .create_test_user("currentuser", "current@test.com")
        .await;
    let other_user = ctx.create_test_user("otheruser", "other@test.com").await;

    let chat = ctx
        .create_group_chat(current_user.id, "Test Chat", vec![other_user.id], None)
        .await;

    let handler = AddOrEditMessageCommandHandler::new(&ctx.app_state);

    let create_cmd = AddOrEditMessageCommand {
        current_user_id: current_user.id,
        message_id: None,
        referenced_message_id: None,
        chat_id: chat.id,
        content: "Original content".to_string(),
        attachments: vec![],
    };

    let create_result = handler
        .handle(create_cmd)
        .await
        .unwrap_or_else(|e| panic!("Create message failed: {:?}", e));

    let edit_cmd = AddOrEditMessageCommand {
        current_user_id: other_user.id,
        message_id: Some(create_result.message.id),
        referenced_message_id: None,
        chat_id: chat.id,
        content: "Hacked content".to_string(),
        attachments: vec![],
    };

    let err = handler
        .handle(edit_cmd)
        .await
        .expect_err("Expected error when editing another user's message");

    assert_err!(err, Error::MessageWasSentByAnotherUser(_));
}

#[tokio::test]
async fn test_add_message_chat_not_found() {
    let ctx = TestContext::new("test_add_message_chat_not_found").await;

    let current_user = ctx
        .create_test_user("currentuser", "current@test.com")
        .await;

    let nonexistent_chat_id = 99999i64;
    let cmd = AddOrEditMessageCommand {
        current_user_id: current_user.id,
        message_id: None,
        referenced_message_id: None,
        chat_id: nonexistent_chat_id,
        content: "Hello".to_string(),
        attachments: vec![],
    };

    let handler = AddOrEditMessageCommandHandler::new(&ctx.app_state);

    let err = handler
        .handle(cmd)
        .await
        .expect_err("Expected ChatNotFound error");

    assert_err!(err, Error::ChatNotFound(_));
}

#[tokio::test]
async fn test_add_message_user_not_member() {
    let ctx = TestContext::new("test_add_message_not_member").await;

    let current_user = ctx
        .create_test_user("currentuser", "current@test.com")
        .await;
    let other_user = ctx.create_test_user("otheruser", "other@test.com").await;
    let non_member = ctx
        .create_test_user("nonmember", "nonmember@test.com")
        .await;

    let chat = ctx
        .create_group_chat(current_user.id, "Test Chat", vec![other_user.id], None)
        .await;

    let cmd = AddOrEditMessageCommand {
        current_user_id: non_member.id,
        message_id: None,
        referenced_message_id: None,
        chat_id: chat.id,
        content: "Hello".to_string(),
        attachments: vec![],
    };

    let handler = AddOrEditMessageCommandHandler::new(&ctx.app_state);

    let err = handler
        .handle(cmd)
        .await
        .expect_err("Expected UserNotMember error");

    assert_err!(err, Error::UserNotMember { .. });
}

#[tokio::test]
async fn test_add_message_insufficient_permissions() {
    let ctx = TestContext::new("test_add_message_insufficient_perms").await;

    let current_user = ctx
        .create_test_user("currentuser", "current@test.com")
        .await;
    let other_user = ctx.create_test_user("otheruser", "other@test.com").await;

    let limited_permissions = ChatPermissions::empty();

    let chat = ctx
        .create_group_chat(
            current_user.id,
            "Test Chat",
            vec![other_user.id],
            Some(limited_permissions),
        )
        .await;

    let cmd = AddOrEditMessageCommand {
        current_user_id: other_user.id,
        message_id: None,
        referenced_message_id: None,
        chat_id: chat.id,
        content: "Hello".to_string(),
        attachments: vec![],
    };

    let handler = AddOrEditMessageCommandHandler::new(&ctx.app_state);

    let err = handler
        .handle(cmd)
        .await
        .expect_err("Expected InsufficientPermissions error");

    assert_err!(err, Error::InsufficientPermissions { .. });
}

#[tokio::test]
async fn test_add_message_saved_in_repository() {
    let ctx = TestContext::new("test_add_message_saved").await;

    let current_user = ctx
        .create_test_user("currentuser", "current@test.com")
        .await;
    let other_user = ctx.create_test_user("otheruser", "other@test.com").await;

    let chat = ctx
        .create_group_chat(current_user.id, "Test Chat", vec![other_user.id], None)
        .await;

    let handler = AddOrEditMessageCommandHandler::new(&ctx.app_state);

    let cmd = valid_command(current_user.id, chat.id);

    let result = handler.handle(cmd).await.unwrap();

    let stored_message = ctx
        .app_state
        .message_repository
        .get_by_id(chat.id, result.message.id)
        .await
        .unwrap_or_else(|e| panic!("Failed to get message: {:?}", e))
        .expect("Message should exist in repository");

    assert_eq!(stored_message.id, result.message.id);
    assert_eq!(stored_message.content, "Hello world");
}

#[tokio::test]
async fn test_edit_nonexistent_message() {
    let ctx = TestContext::new("test_edit_nonexistent").await;

    let current_user = ctx
        .create_test_user("currentuser", "current@test.com")
        .await;
    let other_user = ctx.create_test_user("otheruser", "other@test.com").await;

    let chat = ctx
        .create_group_chat(current_user.id, "Test Chat", vec![other_user.id], None)
        .await;

    let handler = AddOrEditMessageCommandHandler::new(&ctx.app_state);

    let nonexistent_message_id = 99999i64;
    let cmd = AddOrEditMessageCommand {
        current_user_id: current_user.id,
        message_id: Some(nonexistent_message_id),
        referenced_message_id: None,
        chat_id: chat.id,
        content: "Hello".to_string(),
        attachments: vec![],
    };

    let err = handler
        .handle(cmd)
        .await
        .expect_err("Expected MessageNotFound error");

    assert_err!(err, Error::MessageNotFound(_));
}

#[tokio::test]
async fn test_add_message_publishes_event() {
    let ctx = TestContext::new("test_add_message_event").await;

    let current_user = ctx
        .create_test_user("currentuser", "current@test.com")
        .await;
    let other_user = ctx.create_test_user("otheruser", "other@test.com").await;

    let chat = ctx
        .create_group_chat(current_user.id, "Test Chat", vec![other_user.id], None)
        .await;

    let handler = AddOrEditMessageCommandHandler::new(&ctx.app_state);
    let mut receiver = ctx.app_state.event_bus.subscribe();

    let cmd = valid_command(current_user.id, chat.id);

    let _ = handler.handle(cmd).await.unwrap();

    let event = tokio::time::timeout(std::time::Duration::from_secs(1), receiver.recv())
        .await
        .expect("Should receive event")
        .expect("Event should be available");

    assert!(
        matches!(event, DomainEvent::MessageCreated { .. }),
        "Event should be MessageCreated"
    );
}

#[tokio::test]
async fn test_edit_message_publishes_event() {
    let ctx = TestContext::new("test_edit_message_event").await;

    let current_user = ctx
        .create_test_user("currentuser", "current@test.com")
        .await;
    let other_user = ctx.create_test_user("otheruser", "other@test.com").await;

    let chat = ctx
        .create_group_chat(current_user.id, "Test Chat", vec![other_user.id], None)
        .await;

    let handler = AddOrEditMessageCommandHandler::new(&ctx.app_state);

    let create_cmd = AddOrEditMessageCommand {
        current_user_id: current_user.id,
        message_id: None,
        referenced_message_id: None,
        chat_id: chat.id,
        content: "Original".to_string(),
        attachments: vec![],
    };

    let create_result = handler.handle(create_cmd).await.unwrap();

    let mut receiver = ctx.app_state.event_bus.subscribe();

    let edit_cmd = AddOrEditMessageCommand {
        current_user_id: current_user.id,
        message_id: Some(create_result.message.id),
        referenced_message_id: None,
        chat_id: chat.id,
        content: "Edited".to_string(),
        attachments: vec![],
    };

    let _ = handler.handle(edit_cmd).await.unwrap();

    let event = tokio::time::timeout(std::time::Duration::from_secs(1), receiver.recv())
        .await
        .expect("Should receive event")
        .expect("Event should be available");

    assert!(
        matches!(event, DomainEvent::MessageUpdated { .. }),
        "Event should be MessageUpdated"
    );
}

#[tokio::test]
async fn test_add_message_referenced_not_found() {
    let ctx = TestContext::new("test_add_message_ref_not_found").await;

    let current_user = ctx
        .create_test_user("currentuser", "current@test.com")
        .await;
    let other_user = ctx.create_test_user("otheruser", "other@test.com").await;

    let chat = ctx
        .create_group_chat(current_user.id, "Test Chat", vec![other_user.id], None)
        .await;

    let handler = AddOrEditMessageCommandHandler::new(&ctx.app_state);

    let nonexistent_ref_id = 99999i64;
    let cmd = AddOrEditMessageCommand {
        current_user_id: current_user.id,
        message_id: None,
        referenced_message_id: Some(nonexistent_ref_id),
        chat_id: chat.id,
        content: "Hello".to_string(),
        attachments: vec![],
    };

    let err = handler
        .handle(cmd)
        .await
        .expect_err("Expected MessageNotFound error for referenced message");

    assert_err!(err, Error::MessageNotFound(_));
}

#[tokio::test]
async fn test_add_message_with_attachments() {
    let ctx = TestContext::new("test_add_message_with_attachments").await;

    let current_user = ctx
        .create_test_user("currentuser", "current@test.com")
        .await;
    let other_user = ctx.create_test_user("otheruser", "other@test.com").await;

    let chat = ctx
        .create_group_chat(
            current_user.id,
            "Test Chat",
            vec![other_user.id],
            Some(ChatPermissions::all()),
        )
        .await;

    let attachments_handler = CreateCloudAttachmentsCommandHandler::new(&ctx.app_state);
    let attachments_cmd = CreateCloudAttachmentsCommand {
        current_user_id: current_user.id,
        chat_id: chat.id,
        files: vec![
            UploadAttachmentDto {
                id: Some("1".to_string()),
                filename: "test.txt".to_string(),
                filesize: 1024,
            },
            UploadAttachmentDto {
                id: Some("2".to_string()),
                filename: "image.png".to_string(),
                filesize: 2048,
            },
        ],
    };

    let attachments_response = attachments_handler
        .handle(attachments_cmd)
        .await
        .unwrap_or_else(|e| panic!("Create attachments failed: {:?}", e));

    assert_eq!(attachments_response.results.len(), 2);

    let test_content: Vec<u8> = b"test file content".to_vec();
    for upload_filename in &attachments_response.results {
        ctx.app_state
            .s3_service
            .put_object(
                upload_filename.upload_filename.as_str(),
                test_content.clone(),
                "application/octet-stream",
            )
            .await
            .unwrap_or_else(|e| panic!("Failed to upload test file: {:?}", e));
    }

    let message_handler = AddOrEditMessageCommandHandler::new(&ctx.app_state);
    let message_cmd = AddOrEditMessageCommand {
        current_user_id: current_user.id,
        message_id: None,
        referenced_message_id: None,
        chat_id: chat.id,
        content: "Message with attachments".to_string(),
        attachments: vec![
            MessageAttachmentInput {
                uploaded_filename: attachments_response.results[0].upload_filename.clone(),
                filename: "test.txt".to_string(),
            },
            MessageAttachmentInput {
                uploaded_filename: attachments_response.results[1].upload_filename.clone(),
                filename: "image.png".to_string(),
            },
        ],
    };

    let result = message_handler
        .handle(message_cmd)
        .await
        .unwrap_or_else(|e| panic!("Add message with attachments failed: {:?}", e));

    assert_eq!(result.message.content, "Message with attachments");
    assert_eq!(result.attachments.len(), 2);
    assert_eq!(result.attachments[0].filename, "test.txt");
    assert_eq!(result.attachments[1].filename, "image.png");
}

#[tokio::test]
async fn test_add_message_updates_last_message_id() {
    let ctx = TestContext::new("test_add_msg_last_msg_id").await;

    let current_user = ctx
        .create_test_user("currentuser", "current@test.com")
        .await;
    let other_user = ctx.create_test_user("otheruser", "other@test.com").await;

    let chat = ctx
        .create_group_chat(current_user.id, "Test Chat", vec![other_user.id], None)
        .await;

    let result = ctx
        .create_message(chat.id, current_user.id, "Hello world")
        .await;

    let updated_chat = ctx
        .app_state
        .chat_repository
        .get_by_id(chat.id)
        .await
        .expect("Should get chat")
        .expect("Chat should exist");

    assert_eq!(
        updated_chat.last_message_id,
        Some(result.message.id),
        "last_message_id should be updated to the new message id"
    );
}
