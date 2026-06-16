use crate::{
    application::{
        RequestHandler,
        chats::{
            AddChatMemberCommand, AddChatMemberCommandHandler, CreateChatCommand,
            CreateChatCommandHandler,
        },
    },
    assert_err,
    domain::{
        chats::{Chat, ChatMember, ChatPermissions, CreateGroupChatRequest},
        events::DomainEvent,
    },
    error::Error,
    tests::common::TestContext,
};

#[tokio::test]
async fn test_add_chat_member_success() {
    let ctx = TestContext::new("test_add_member_success").await;
    let handler = AddChatMemberCommandHandler::new(&ctx.app_state);

    let owner = ctx.create_test_user("owner", "owner@test.com").await;
    let member = ctx.create_test_user("member", "member@test.com").await;
    let new_user = ctx.create_test_user("newuser", "new@test.com").await;

    let chat_handler = CreateChatCommandHandler::new(&ctx.app_state);
    let chat = chat_handler
        .handle(CreateChatCommand {
            current_user_id: owner.id,
            name: "Test Group".to_string(),
            members: vec![member.id],
            permissions: None,
        })
        .await
        .expect("Failed to create chat");

    let cmd = AddChatMemberCommand {
        current_user_id: owner.id,
        chat_id: chat.id,
        user_id: new_user.id,
    };

    handler.handle(cmd).await.expect("Should succeed");
}

#[tokio::test]
async fn test_add_chat_member_not_member() {
    let ctx = TestContext::new("test_add_member_not_member").await;
    let handler = AddChatMemberCommandHandler::new(&ctx.app_state);

    let owner = ctx.create_test_user("owner", "owner@test.com").await;
    let member = ctx.create_test_user("member", "member@test.com").await;
    let new_user = ctx.create_test_user("newuser", "new@test.com").await;

    let chat_handler = CreateChatCommandHandler::new(&ctx.app_state);
    let chat = chat_handler
        .handle(CreateChatCommand {
            current_user_id: owner.id,
            name: "Test Group".to_string(),
            members: vec![member.id],
            permissions: None,
        })
        .await
        .expect("Failed to create chat");

    let cmd = AddChatMemberCommand {
        current_user_id: 99999i64,
        chat_id: chat.id,
        user_id: new_user.id,
    };

    let err = handler
        .handle(cmd)
        .await
        .expect_err("Should fail - not member");

    assert_err!(err, Error::UserNotMember { .. });
}

#[tokio::test]
async fn test_add_chat_member_dm_not_supported() {
    let ctx = TestContext::new("test_add_member_dm").await;
    let handler = AddChatMemberCommandHandler::new(&ctx.app_state);

    let user1 = ctx.create_test_user("user1", "user1@test.com").await;
    let user2 = ctx.create_test_user("user2", "user2@test.com").await;
    let user3 = ctx.create_test_user("user3", "user3@test.com").await;
    let dm_chat = ctx.get_or_create_dm_chat(user1.id, user2.id).await;

    let cmd = AddChatMemberCommand {
        current_user_id: user1.id,
        chat_id: dm_chat.id,
        user_id: user3.id,
    };

    let err = handler.handle(cmd).await.expect_err("Should fail - DM");

    assert_err!(err, Error::ChatTypeNotSupported { .. });
}

#[tokio::test]
async fn test_add_chat_member_no_permission() {
    let ctx = TestContext::new("test_add_member_no_permission").await;
    let handler = AddChatMemberCommandHandler::new(&ctx.app_state);

    let owner = ctx.create_test_user("owner", "owner@test.com").await;
    let member = ctx.create_test_user("member", "member@test.com").await;
    let new_user = ctx.create_test_user("newuser", "new@test.com").await;

    let no_add_members_perms = ChatPermissions::SEND_MESSAGES | ChatPermissions::SEND_FILES;
    let chat = Chat::create_group_dm(CreateGroupChatRequest {
        id: ctx.app_state.id_gen.gen_id().await,
        owner_id: owner.id,
        name: "Test Group".to_string(),
        permissions: Some(no_add_members_perms),
        members: vec![
            ChatMember::from(owner.clone()),
            ChatMember::from(member.clone()),
        ],
    });

    ctx.app_state
        .chat_repository
        .save(&chat)
        .await
        .expect("Failed to save chat");

    let cmd = AddChatMemberCommand {
        current_user_id: member.id,
        chat_id: chat.id,
        user_id: new_user.id,
    };

    let err = handler
        .handle(cmd)
        .await
        .expect_err("Should fail - no permission");

    assert_err!(err, Error::InsufficientPermissions { .. });
}

#[tokio::test]
async fn test_add_chat_member_already_member() {
    let ctx = crate::tests::common::TestContext::new("test_add_member_already").await;
    let handler = AddChatMemberCommandHandler::new(&ctx.app_state);

    let owner = ctx.create_test_user("owner", "owner@test.com").await;
    let member = ctx.create_test_user("member", "member@test.com").await;

    let chat_handler = CreateChatCommandHandler::new(&ctx.app_state);
    let chat = chat_handler
        .handle(CreateChatCommand {
            current_user_id: owner.id,
            name: "Test Group".to_string(),
            members: vec![member.id],
            permissions: None,
        })
        .await
        .expect("Failed to create chat");

    let cmd = AddChatMemberCommand {
        current_user_id: owner.id,
        chat_id: chat.id,
        user_id: member.id,
    };

    let err = handler
        .handle(cmd)
        .await
        .expect_err("Should fail - already member");

    assert_err!(err, Error::UserAlreadyMember { .. });
}

#[tokio::test]
async fn test_add_chat_member_chat_not_found() {
    let ctx = crate::tests::common::TestContext::new("test_add_member_not_found").await;
    let handler = AddChatMemberCommandHandler::new(&ctx.app_state);

    let owner = ctx.create_test_user("owner", "owner@test.com").await;

    let cmd = AddChatMemberCommand {
        current_user_id: owner.id,
        chat_id: 99999i64,
        user_id: owner.id,
    };

    let err = handler
        .handle(cmd)
        .await
        .expect_err("Should fail - not found");

    assert_err!(err, Error::ChatNotFound(_));
}

#[tokio::test]
async fn test_add_chat_member_user_not_found() {
    let ctx = crate::tests::common::TestContext::new("test_add_member_user_not_found").await;
    let handler = AddChatMemberCommandHandler::new(&ctx.app_state);

    let owner = ctx.create_test_user("owner", "owner@test.com").await;
    let member = ctx.create_test_user("member", "member@test.com").await;

    let chat_handler = CreateChatCommandHandler::new(&ctx.app_state);
    let chat = chat_handler
        .handle(CreateChatCommand {
            current_user_id: owner.id,
            name: "Test Group".to_string(),
            members: vec![member.id],
            permissions: None,
        })
        .await
        .expect("Failed to create chat");

    let cmd = AddChatMemberCommand {
        current_user_id: owner.id,
        chat_id: chat.id,
        user_id: 99999i64,
    };

    let err = handler
        .handle(cmd)
        .await
        .expect_err("Should fail - user not found");

    assert_err!(err, Error::UserNotFound(_));
}
