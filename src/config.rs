use dotenvy::dotenv;
use once_cell::sync::OnceCell;
use std::env;

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
    pub access_token_expiration_seconds: u64,
    pub keys_directory: String,
}

#[derive(Clone)]
pub struct S3Config {
    pub access_key: String,
    pub secret_key: String,
    pub service_url: String,
    pub force_path_style: bool,
    pub bucket_name: String,
    pub region: String,
}

#[derive(Clone)]
pub struct Config {
    pub port: u16,
    pub scylla_node: String,
    pub smtp: SmtpConfig,
    pub auth: AuthConfig,
    pub s3: S3Config,
    pub nats_url: String,
}

pub static CONFIG: OnceCell<Config> = OnceCell::new();

fn init_config() -> Config {
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
            access_token_expiration_seconds: env::var("ACCESS_TOKEN_EXPIRATION_SECONDS")
                .unwrap_or("3600".to_string())
                .parse()
                .expect("Failed to parse access token expiration"),
            keys_directory: env::var("KEYS_DIRECTORY").unwrap_or("keys".to_string()),
        },
        nats_url: env::var("NATS_URL").unwrap_or("nats://127.0.0.1:4222".to_string()),
        s3: S3Config {
            access_key: env::var("AWS_ACCESS_KEY_ID").expect("AWS_ACCESS_KEY_ID required"),
            secret_key: env::var("AWS_SECRET_ACCESS_KEY").expect("AWS_SECRET_ACCESS_KEY required"),
            service_url: env::var("AWS_SERVICE_URL").expect("AWS_SERVICE_URL required"),
            force_path_style: env::var("AWS_FORCE_PATH_STYLE")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            bucket_name: env::var("AWS_BUCKET_NAME").expect("AWS_BUCKET_NAME required"),
            region: env::var("AWS_REGION").unwrap_or("us-east-1".to_string()),
        },
    }
}

pub fn config() -> &'static Config {
    CONFIG.get_or_init(init_config)
}
