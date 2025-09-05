use async_trait::async_trait;

use crate::config::AppConfig;
use crate::models::WebhookMessage;
use crate::services::PermataCallbackStatusClient;
use crate::utils::error::Result;
use crate::providers::logging::StructuredLogger;

#[async_trait]
pub trait WebhookProcessorTrait {
    async fn process_webhook(&self, webhook: WebhookMessage, request_id: &str) -> Result<()>;
}

#[derive(Clone)]
pub struct WebhookProcessor {
    permata_client: PermataCallbackStatusClient,
}

impl WebhookProcessor {
    pub fn new(config: AppConfig) -> Result<Self> {
        let permata_client = PermataCallbackStatusClient::new(config)?;
        Ok(Self {
            permata_client,
        })
    }

    fn is_authentication_error(&self, error: &crate::utils::error::AppError) -> bool {
        match error {
            crate::utils::error::AppError::AuthenticationFailed { .. } => true,
            crate::utils::error::AppError::Hmac(_) => true, // HMAC errors often indicate auth issues
            _ => {
                let error_message = format!("{}", error);
                error_message.contains("Login failed") ||
                error_message.contains("Token") ||
                error_message.contains("authentication") ||
                error_message.contains("unauthorized") ||
                error_message.contains("Unauthorized") ||
                error_message.contains("401")
            }
        }
    }

}

#[async_trait]
impl WebhookProcessorTrait for WebhookProcessor {
    async fn process_webhook(&self, webhook: WebhookMessage, request_id: &str) -> Result<()> {
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
            Ok(response) => {
                StructuredLogger::log_info(
                    "Webhook processed successfully",
                    Some(request_id),
                    Some(request_id),
                    Some(serde_json::json!({
                        "permata_status_code": response.status_code,
                        "permata_status_desc": response.status_desc
                    })),
                );
                Ok(())
            }
            Err(e) => {
                // Check if the error is authentication-related
                if self.is_authentication_error(&e) {
                    StructuredLogger::log_error(
                        &format!("Authentication failed - skipping webhook forwarding for request {}: {}", request_id, e),
                        Some(request_id),
                        Some(request_id),
                    );
                    
                    // Log additional details for debugging
                    StructuredLogger::log_info(
                        "Webhook received but not forwarded due to authentication issues",
                        Some(request_id),
                        Some(request_id),
                        Some(serde_json::json!({
                            "reason": "authentication_failed",
                            "body_size": webhook.body.len(),
                            "headers_count": webhook.headers.len(),
                            "error_type": "auth"
                        })),
                    );
                    
                    // Return success to avoid retrying failed auth
                    // The webhook is received but not forwarded due to auth issues
                    Ok(())
                } else {
                    StructuredLogger::log_error(
                        &format!("Failed to process webhook for request {}: {}", request_id, e),
                        Some(request_id),
                        Some(request_id),
                    );
                    Err(e)
                }
            }
        }
    }
}

