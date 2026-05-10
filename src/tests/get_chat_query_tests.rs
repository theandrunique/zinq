use crate::{
    application::{
        RequestHandler,
        chats::{CreateChatCommand, CreateChatCommandHandler},
        chats::{GetChatQuery, GetChatQueryHandler},
    },
    assert_err,
    error::Error,
    tests::common::TestContext,
};

#[tokio::test]
async fn test_get_chat_success() {
    let ctx = TestContext::new("test_get_chat_success").await;

    let owner = ctx.create_test_user("owner", "owner@test.com").await;
    let member1 = ctx.create_test_user("member1", "member1@test.com").await;
    let member2 = ctx.create_test_user("member2", "member2@test.com").await;
    let member3 = ctx.create_test_user("member3", "member3@test.com").await;

    let chat = ctx
        .create_group_chat(
            owner.id,
            "Test Group",
            vec![member1.id, member2.id, member3.id],
            None,
        )
        .await;

    let query_handler = GetChatQueryHandler::new(&ctx.app_state);
    let query = GetChatQuery {
        current_user_id: owner.id,
        chat_id: chat.id,
    };

    let result = query_handler.handle(query).await.expect("Should succeed");

    assert_eq!(result.id, chat.id);
    assert_eq!(result.name, Some("Test Group".to_string()));
    assert_eq!(result.members.len(), 4);

    let member_ids: Vec<i64> = result.members.iter().map(|m| m.user_id).collect();
    assert!(member_ids.contains(&owner.id));
    assert!(member_ids.contains(&member1.id));
    assert!(member_ids.contains(&member2.id));
    assert!(member_ids.contains(&member3.id));
}

#[tokio::test]
async fn test_get_chat_not_found() {
    let ctx = TestContext::new("test_get_chat_not_found").await;

    let query_handler = GetChatQueryHandler::new(&ctx.app_state);
    let query = GetChatQuery {
        current_user_id: 1,
        chat_id: 99999,
    };

    let err = query_handler
        .handle(query)
        .await
        .expect_err("Should fail - chat not found");

    assert_err!(err, Error::ChatNotFound(_));
}

#[tokio::test]
async fn test_get_chat_not_member() {
    let ctx = TestContext::new("test_get_chat_not_member").await;

    let owner = ctx.create_test_user("owner", "owner@test.com").await;
    let member = ctx.create_test_user("member", "member@test.com").await;
    let stranger = ctx.create_test_user("stranger", "stranger@test.com").await;
    let chat = ctx
        .create_group_chat(owner.id, "Test Group", vec![member.id], None)
        .await;

    let query_handler = GetChatQueryHandler::new(&ctx.app_state);
    let query = GetChatQuery {
        current_user_id: stranger.id,
        chat_id: chat.id,
    };

    let err = query_handler
        .handle(query)
        .await
        .expect_err("Should fail - not member");

    assert_err!(err, Error::UserNotMember { .. });
}
