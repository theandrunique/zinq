use crate::application::chats::{CreateChatCommand, CreateChatCommandHandler};
use crate::{
    application::{
        RequestHandler,
        chats::{DeleteChatMemberCommand, DeleteChatMemberCommandHandler},
    },
    domain::chats::Chat,
    tests::common::TestContext,
};

#[tokio::test]
async fn test_delete_chat_member_success() {
    let ctx = TestContext::new("test_delete_member_success").await;
    let handler = DeleteChatMemberCommandHandler::new(&ctx.app_state);

    let owner = ctx.create_test_user("owner", "owner@test.com").await;
    let member = ctx.create_test_user("member", "member@test.com").await;

    let chat_handler = CreateChatCommandHandler::new(&ctx.app_state);
    let chat = chat_handler
        .handle(CreateChatCommand {
            current_user_id: owner.id,
            name: "Test Group".to_string(),
            members: vec![member.id],
        })
        .await
        .expect("Failed to create chat");

    let cmd = DeleteChatMemberCommand {
        current_user_id: owner.id,
        chat_id: chat.id,
        user_id: member.id,
    };

    handler.handle(cmd).await.expect("Should succeed");
}

#[tokio::test]
async fn test_delete_chat_member_not_member() {
    let ctx = TestContext::new("test_delete_member_not_member").await;
    let handler = DeleteChatMemberCommandHandler::new(&ctx.app_state);

    let owner = ctx.create_test_user("owner", "owner@test.com").await;
    let member = ctx.create_test_user("member", "member@test.com").await;

    let chat_handler = CreateChatCommandHandler::new(&ctx.app_state);
    let chat = chat_handler
        .handle(CreateChatCommand {
            current_user_id: owner.id,
            name: "Test Group".to_string(),
            members: vec![member.id],
        })
        .await
        .expect("Failed to create chat");

    let cmd = DeleteChatMemberCommand {
        current_user_id: 99999i64,
        chat_id: chat.id,
        user_id: member.id,
    };

    let err = handler
        .handle(cmd)
        .await
        .expect_err("Should fail - not member");

    let err_str = format!("{:?}", err);
    assert!(
        err_str.contains("UserNotMember"),
        "Error should be UserNotMember, got: {}",
        err_str
    );
}

#[tokio::test]
async fn test_delete_chat_member_dm_not_supported() {
    let ctx = TestContext::new("test_delete_member_dm").await;
    let handler = DeleteChatMemberCommandHandler::new(&ctx.app_state);

    let user1 = ctx.create_test_user("user1", "user1@test.com").await;
    let user2 = ctx.create_test_user("user2", "user2@test.com").await;

    let dm_chat = Chat::create_dm(
        ctx.app_state.id_gen.gen_id().await,
        user1.clone(),
        user2.clone(),
    );

    ctx.app_state
        .chat_repotisory
        .save(dm_chat.clone())
        .await
        .expect("Failed to save DM chat");

    let cmd = DeleteChatMemberCommand {
        current_user_id: user1.id,
        chat_id: dm_chat.id,
        user_id: user2.id,
    };

    let err = handler.handle(cmd).await.expect_err("Should fail - DM");

    let err_str = format!("{:?}", err);
    assert!(
        err_str.contains("ChatTypeNotSupported"),
        "Error should be ChatTypeNotSupported, got: {}",
        err_str
    );
}

#[tokio::test]
async fn test_delete_chat_member_no_permission() {
    let ctx = TestContext::new("test_delete_member_no_permission").await;
    let handler = DeleteChatMemberCommandHandler::new(&ctx.app_state);

    let owner = ctx.create_test_user("owner", "owner@test.com").await;
    let member = ctx.create_test_user("member", "member@test.com").await;

    let chat_handler = CreateChatCommandHandler::new(&ctx.app_state);
    let chat = chat_handler
        .handle(CreateChatCommand {
            current_user_id: owner.id,
            name: "Test Group".to_string(),
            members: vec![member.id],
        })
        .await
        .expect("Failed to create chat");

    let cmd = DeleteChatMemberCommand {
        current_user_id: member.id,
        chat_id: chat.id,
        user_id: member.id,
    };

    let err = handler
        .handle(cmd)
        .await
        .expect_err("Should fail - no permission");

    let err_str = format!("{:?}", err);
    assert!(
        err_str.contains("InsufficientPermissions"),
        "Error should be InsufficientPermissions, got: {}",
        err_str
    );
}

#[tokio::test]
async fn test_delete_chat_member_target_not_found() {
    let ctx = TestContext::new("test_delete_target_not_found").await;
    let handler = DeleteChatMemberCommandHandler::new(&ctx.app_state);

    let owner = ctx.create_test_user("owner", "owner@test.com").await;
    let member = ctx.create_test_user("member", "member@test.com").await;

    let chat_handler = CreateChatCommandHandler::new(&ctx.app_state);
    let chat = chat_handler
        .handle(CreateChatCommand {
            current_user_id: owner.id,
            name: "Test Group".to_string(),
            members: vec![member.id],
        })
        .await
        .expect("Failed to create chat");

    let cmd = DeleteChatMemberCommand {
        current_user_id: owner.id,
        chat_id: chat.id,
        user_id: member.id + 99999,
    };

    let err = handler
        .handle(cmd)
        .await
        .expect_err("Should fail - target not found");

    let err_str = format!("{:?}", err);
    assert!(
        err_str.contains("UserNotMember"),
        "Error should be UserNotMember, got: {}",
        err_str
    );
}

#[tokio::test]
async fn test_delete_chat_member_chat_not_found() {
    let ctx = TestContext::new("test_delete_chat_not_found").await;
    let handler = DeleteChatMemberCommandHandler::new(&ctx.app_state);

    let owner = ctx.create_test_user("owner", "owner@test.com").await;

    let cmd = DeleteChatMemberCommand {
        current_user_id: owner.id,
        chat_id: 99999i64,
        user_id: owner.id,
    };

    let err = handler
        .handle(cmd)
        .await
        .expect_err("Should fail - not found");

    let err_str = format!("{:?}", err);
    assert!(
        err_str.contains("ChatNotFound"),
        "Error should be ChatNotFound, got: {}",
        err_str
    );
}

#[tokio::test]
async fn test_delete_chat_member_publishes_event() {
    let ctx = TestContext::new("test_delete_member_event").await;
    let handler = DeleteChatMemberCommandHandler::new(&ctx.app_state);

    let owner = ctx.create_test_user("owner", "owner@test.com").await;
    let member = ctx.create_test_user("member", "member@test.com").await;

    let chat_handler = CreateChatCommandHandler::new(&ctx.app_state);
    let chat = chat_handler
        .handle(CreateChatCommand {
            current_user_id: owner.id,
            name: "Test Group".to_string(),
            members: vec![member.id],
        })
        .await
        .expect("Failed to create chat");

    let mut receiver = ctx.app_state.event_bus.subscribe();

    let cmd = DeleteChatMemberCommand {
        current_user_id: owner.id,
        chat_id: chat.id,
        user_id: member.id,
    };

    let _ = handler.handle(cmd).await.expect("Should succeed");

    let event = tokio::time::timeout(std::time::Duration::from_secs(1), receiver.recv())
        .await
        .expect("Should receive event")
        .expect("Event should be available");

    let event_str = format!("{:?}", event);
    assert!(
        event_str.contains("ChatMemberRemoved"),
        "Event should be ChatMemberRemoved, got: {}",
        event_str
    );
}
