use serde::{Deserialize, Serialize};

use crate::utils::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub webclient: WebClientConfig,
    pub permata_bank_login: PermataBankLoginConfig,
    pub permata_bank_webhook: PermataBankWebhookConfig,
    pub token_scheduler: SchedulerConfig,
    pub logger: LoggerConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub listen_host: String,
    pub listen_port: u16,
    pub webhook_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebClientConfig {
    pub timeout: u64,
    pub max_retries: u32,
    pub retry_delay: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermataBankLoginConfig {
    pub permata_static_key: String,
    pub api_key: String,
    pub token_url: String,
    pub username: String,
    pub password: String,
    pub login_payload: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermataBankWebhookConfig {
    pub callbackstatus_url: String,
    pub organizationname: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerConfig {
    pub periodic_interval_mins: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggerConfig {
    pub dir: String,
    pub file_name: String,
    pub max_backups: u32,
    pub max_size: u32,
    pub max_age: u32,
    pub compress: bool,
    pub local_time: bool,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let settings = config::Config::builder()
            .add_source(config::File::with_name("config.yaml"))
            .add_source(config::Environment::with_prefix("APP"))
            .build()?;

        Ok(settings.try_deserialize()?)
    }
}