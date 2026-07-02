use crate::{
    application::{
        RequestHandler,
        chats::{CreateChatCommand, CreateChatCommandHandler},
        chats::{GetUserChatsQuery, GetUserChatsQueryHandler},
    },
    domain::chats::{Chat, ChatMember, ChatType},
    tests::common::TestContext,
};

#[tokio::test]
async fn test_get_user_chats_empty() {
    let ctx = TestContext::new("test_get_user_chats_empty").await;

    let user = ctx.create_test_user("testuser", "user@test.com").await;

    let query_handler = GetUserChatsQueryHandler::new(&ctx.app_state);
    let query = GetUserChatsQuery {
        current_user_id: user.id,
    };

    let result = query_handler.handle(query).await.expect("Should succeed");
    assert!(result.is_empty());
}

#[tokio::test]
async fn test_get_user_chats_returns_chats() {
    let ctx = TestContext::new("test_get_user_chats_returns_chats").await;

    let current_user = ctx.create_test_user("owner", "owner@test.com").await;
    let member1 = ctx.create_test_user("member1", "member1@test.com").await;
    let member2 = ctx.create_test_user("member2", "member2@test.com").await;
    let member3 = ctx.create_test_user("member3", "member3@test.com").await;

    let group_chat = ctx
        .create_group_chat(
            current_user.id,
            "Test Group",
            vec![member1.id, member2.id],
            None,
        )
        .await;
    let _ = ctx
        .create_message(group_chat.id, current_user.id, "Hello group")
        .await;

    let dm_chat1 = ctx.get_or_create_dm_chat(current_user.id, member2.id).await;
    let _ = ctx
        .create_message(dm_chat1.id, current_user.id, "Hello DM 1")
        .await;

    let dm_chat2 = ctx.get_or_create_dm_chat(current_user.id, member3.id).await;
    let _ = ctx
        .create_message(dm_chat2.id, current_user.id, "Hello DM 2")
        .await;

    let query_handler = GetUserChatsQueryHandler::new(&ctx.app_state);
    let query = GetUserChatsQuery {
        current_user_id: current_user.id,
    };

    let result = query_handler.handle(query).await.expect("Should succeed");

    assert_eq!(result.len(), 3, "Should return 3 chats (1 group + 2 DM)");

    for chat in &result {
        assert!(
            chat.last_message_id.is_some(),
            "All returned chats should have last_message_id set"
        );

        match chat.chat_type {
            ChatType::GroupDm => {
                assert!(
                    chat.members.len() == 1 && chat.members[0].user_id == current_user.id,
                    "GroupDm chat should have loaded only current user"
                );
            }
            ChatType::Dm => {
                assert!(
                    !chat.members.is_empty(),
                    "DM chat should have members loaded"
                );
                let member_ids: Vec<i64> = chat.members.iter().map(|m| m.user_id).collect();
                assert!(
                    member_ids.contains(&current_user.id),
                    "DM should contain current user"
                );
                assert!(
                    member_ids.len() >= 1 && member_ids.len() <= 2,
                    "DM chat should have 1 or 2 members (self-DM or with other user)"
                );
            }
        }
    }
}

#[tokio::test]
async fn test_get_user_chats_dm_without_message_filtered() {
    let ctx = TestContext::new("test_get_user_chats_dm_no_msg").await;

    let user1 = ctx.create_test_user("user1", "user1@test.com").await;
    let user2 = ctx.create_test_user("user2", "user2@test.com").await;
    let _dm_chat = ctx.get_or_create_dm_chat(user1.id, user2.id).await;

    let query_handler = GetUserChatsQueryHandler::new(&ctx.app_state);
    let query = GetUserChatsQuery {
        current_user_id: user1.id,
    };

    let result = query_handler.handle(query).await.expect("Should succeed");
    assert!(result.is_empty());
}
