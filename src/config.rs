use dotenvy::dotenv;
use std::env;

use tokio::sync::OnceCell;

#[derive(Clone)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub from: String,
}

#[derive(Clone)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub jwt_expiration_seconds: usize,
}

pub struct Config {
    pub port: u16,
    pub scylla_node: String,
    pub smtp: SmtpConfig,
    pub auth: AuthConfig,
}

pub static CONFIG: OnceCell<Config> = OnceCell::const_new();

async fn init_config() -> Config {
    dotenv().ok();

    Config {
        port: env::var("PORT")
            .unwrap_or("3000".to_string())
            .parse()
            .expect("Failed to parse port"),
        scylla_node: env::var("SCYLLA_NODE").unwrap_or("127.0.0.1:9042".to_string()),
        smtp: SmtpConfig {
            host: env::var("SMTP_HOST").unwrap_or("smtp.example.com".to_string()),
            port: env::var("SMTP_PORT")
                .unwrap_or("587".to_string())
                .parse()
                .expect("Failed to parse smtp port"),
            username: env::var("SMTP_USERNAME").unwrap_or_default(),
            password: env::var("SMTP_PASSWORD").unwrap_or_default(),
            from: env::var("SMTP_FROM").unwrap_or("noreply@example.com".to_string()),
        },
        auth: AuthConfig {
            jwt_secret: env::var("JWT_SECRET").unwrap_or("default_secret_change_me".to_string()),
            jwt_expiration_seconds: env::var("JWT_EXPIRATION_SECONDS")
                .unwrap_or("3600".to_string())
                .parse()
                .expect("Failed to parse jwt expiration"),
        },
    }
}

pub async fn config() -> &'static Config {
    CONFIG.get_or_init(init_config).await
}