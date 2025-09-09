use async_trait::async_trait;

use crate::config::AppConfig;
use crate::models::WebhookMessage;
use crate::services::{PermataCallbackStatusClient, TelegramAlertService};
use crate::utils::error::Result;
use crate::providers::logging::StructuredLogger;

#[derive(Debug, Clone)]
pub struct WebhookResponse {
    pub http_status: u16,
    pub body: String,
}

#[async_trait]
pub trait WebhookProcessorTrait {
    async fn process_webhook(&self, webhook: WebhookMessage, request_id: &str) -> Result<WebhookResponse>;
}

#[derive(Clone)]
pub struct WebhookProcessor {
    permata_client: PermataCallbackStatusClient,
    config: AppConfig,
}

impl WebhookProcessor {
    pub fn new(config: AppConfig) -> Result<Self> {
        let permata_client = PermataCallbackStatusClient::new(config.clone())?;
        Ok(Self {
            permata_client,
            config,
        })
    }

    pub async fn shutdown(&self) {
        StructuredLogger::log_info(
            "Shutting down WebhookProcessor",
            None,
            None,
            None,
        );
        self.permata_client.shutdown().await;
    }
}

#[async_trait]
impl WebhookProcessorTrait for WebhookProcessor {
    async fn process_webhook(&self, webhook: WebhookMessage, request_id: &str) -> Result<WebhookResponse> {
        StructuredLogger::log_info(
            "Processing webhook for Permata Bank",
            Some(request_id),
            Some(request_id),
            Some(serde_json::json!({
                "body_size": webhook.body.len(),
                "headers_count": webhook.headers.len()
            })),
        );

        // Send webhook to Permata Bank callback status URL
        match self.permata_client.send_webhook_with_context(&webhook.body, request_id, Some(request_id), Some(request_id)).await {
            Ok(http_response) => {
                // Return langsung HTTP response dari Permata Bank
                Ok(WebhookResponse {
                    http_status: http_response.status_code,
                    body: http_response.body,
                })
            }
            Err(e) => {
                let error_message = format!("Failed to process webhook for request {}: {}", request_id, e);
                
                StructuredLogger::log_error(
                    &error_message,
                    Some(request_id),
                    Some(request_id),
                );
                
                // Send telegram alert for webhook failures
                if let Ok(telegram_service) = TelegramAlertService::new(self.config.clone()) {
                    telegram_service.send_error_alert(
                        &error_message,
                        Some(request_id)
                    );
                }
                
                // Check if this is an authentication error - handle gracefully
                let error_msg = e.to_string();
                if error_msg.contains("Authentication failed") || error_msg.contains("Login failed") {                    
                    // Return a 401 Unauthorized to indicate upstream authentication issues
                    Ok(WebhookResponse {
                        http_status: 401,
                        body: format!(r#"{{"error": "Authentication failed", "message": "{}"}}"#, error_msg),
                    })
                } else {
                    Err(e)
                }
            }
        }
    }
}

