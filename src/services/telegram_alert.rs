use reqwest::Client;
use serde_json::json;
use std::time::Duration;

use crate::config::AppConfig;
use crate::utils::error::Result;
use crate::providers::StructuredLogger;

#[derive(Clone)]
pub struct TelegramAlertService {
    client: Client,
    config: AppConfig,
}

impl TelegramAlertService {
    pub fn new(config: AppConfig) -> Result<Self> {
        let timeout = Duration::from_secs(config.webclient.timeout);
        let client = Client::builder()
            .timeout(timeout)
            .build()?;

        Ok(Self { client, config })
    }

    pub fn send_error_alert(&self, error_message: &str, request_id: Option<&str>) {
        let telegram_config = self.config.telegram_alert.clone();
        let client = self.client.clone();
        let error_message = error_message.to_string();
        let request_id = request_id.map(|s| s.to_string());
        
        // Spawn async task untuk non-blocking execution
        tokio::spawn(async move {
            let formatted_message = match request_id {
                Some(req_id) => format!(
                    "{} [request-id: {}] {}",
                    telegram_config.alert_message_prefix,
                    req_id,
                    error_message
                ),
                None => format!(
                    "{} {}",
                    telegram_config.alert_message_prefix,
                    error_message
                ),
            };

            let payload = json!({
                "chat_id": telegram_config.chat_id,
                "message_thread_id": telegram_config.message_thread_id,
                "text": formatted_message
            });

            match client
                .post(&telegram_config.api_url)
                .header("Content-Type", "application/json")
                .json(&payload)
                .send()
                .await
            {
                Ok(response) => {
                    if response.status().is_success() {
                        StructuredLogger::log_info(
                            &format!("Telegram alert sent successfully: {}", formatted_message), 
                            None, None, None
                        );
                    } else {
                        let status = response.status();
                        let error_text = response.text().await.unwrap_or_default();
                        StructuredLogger::log_error(&format!(
                            "Failed to send Telegram alert. Status: {}, Error: {}",
                            status, error_text
                        ), None, None);
                    }
                }
                Err(e) => {
                    StructuredLogger::log_error(&format!(
                        "Failed to send Telegram alert: {}",
                        e
                    ), None, None);
                }
            }
        });
    }
}
