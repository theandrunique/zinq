#[cfg(test)]
mod integration_tests {
    use crate::application::{RegisterComandHandler, RegisterCommand, tests::common::TestContext};

    fn valid_command() -> RegisterCommand {
        RegisterCommand {
            username: "testuser".to_string(),
            password: "Test123!".to_string(),
            global_name: "Test User".to_string(),
            email: "test@example.com".to_string(),
        }
    }

    #[tokio::test]
    async fn test_register_command_success() {
        let ctx = TestContext::new("test_reggister_command_success").await;

        let handler = RegisterComandHandler::new(&ctx.app_state);

        let user = handler
            .handle(valid_command())
            .await
            .unwrap_or_else(|e| panic!("Register command failed: {:?}", e));

        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.global_name, "Test User");
        assert!(user.is_active);
        assert!(!user.is_email_verified);
    }

    #[tokio::test]
    async fn test_register_duplicate_username() {
        let ctx = TestContext::new("test_register_duplicate_username").await;
        let handler = RegisterComandHandler::new(&ctx.app_state);

        let cmd = valid_command();

        let first = handler.handle(cmd.clone()).await;
        assert!(first.is_ok());

        let err = handler
            .handle(cmd)
            .await
            .expect_err("Expected error when registering duplicate username");
        dbg!(&err);
    }

    #[tokio::test]
    async fn test_register_duplicate_email() {
        let ctx = TestContext::new("test_register_duplicate_email").await;
        let handler = RegisterComandHandler::new(&ctx.app_state);

        let mut cmd = valid_command();

        cmd.username = "test1".to_string();

        let first = handler.handle(cmd.clone()).await;
        assert!(first.is_ok());

        cmd.username = "test2".to_string();

        let err = handler
            .handle(cmd)
            .await
            .expect_err("Expected error when registering duplicate email");

        dbg!(&err);
    }

    #[tokio::test]
    async fn test_user_saved_in_repository() {
        let ctx = TestContext::new("test_user_saved_in_repository").await;
        let handler = RegisterComandHandler::new(&ctx.app_state);

        let user = handler.handle(valid_command()).await.unwrap();

        let stored_user = ctx
            .app_state
            .user_repository
            .get_by_id(user.id)
            .await
            .unwrap_or_else(|e| panic!("Failed to get user by id: {:?}", e))
            .expect("User should exist in repository");
        assert_eq!(stored_user.username, user.username);
    }
}
