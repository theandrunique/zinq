use dotenvy::dotenv;
use std::env;

use tokio::sync::OnceCell;

pub struct Config {
    pub port: u16,
    pub scylla_node: String,
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
    }
}

pub async fn config() -> &'static Config {
    CONFIG.get_or_init(init_config).await
}
