use std::time::Duration;

use reqwest::Client;
use tokio::time::sleep;

use crate::config::{AppConfig, PermataBankWebhookConfig};
use crate::services::LoginHandler;
use crate::models::PermataWebhookResponse;
use crate::providers::StructuredLogger;
use crate::utils::{error::Result, generate_signature, compact_json};

#[derive(Clone)]
pub struct PermataCallbackStatusClient {
    client: Client,
    config: AppConfig,
    login_handler: LoginHandler,
}

impl PermataCallbackStatusClient {
    pub fn new(config: AppConfig) -> Result<Self> {
        let timeout = Duration::from_secs(config.webclient.timeout);
        let client = Client::builder()
            .timeout(timeout)
            .build()?;

        let login_handler = LoginHandler::new(config.clone())?;

        Ok(Self {
            client,
            config,
            login_handler,
        })
    }

    pub async fn send_webhook(&self, webhook_body: &str, request_id: &str) -> Result<PermataWebhookResponse> {
        self.send_webhook_with_context(webhook_body, request_id, Some(request_id), Some(request_id)).await
    }

    pub async fn send_webhook_with_context(&self, webhook_body: &str, request_id: &str, unique_id: Option<&str>, x_request_id: Option<&str>) -> Result<PermataWebhookResponse> {
        let webhook_config = &self.config.permata_bank_webhook;
        let webclient_config = &self.config.webclient;
        
        let mut last_error = None;
        
        for attempt in 1..=webclient_config.max_retries {
            match self.make_webhook_request_with_context(webhook_config, webhook_body, request_id, unique_id, x_request_id).await {
                Ok(response) => {
                    StructuredLogger::log_info(
                        &format!("Webhook sent successfully on attempt {} for request {}", attempt, request_id),
                        unique_id,
                        x_request_id,
                        None,
                    );
                    return Ok(response);
                }
                Err(e) => {
                    // Check if this is an authentication error - don't retry these
                    if self.is_authentication_error(&e) {
                        StructuredLogger::log_error(
                            &format!("Authentication failed for request {} - not retrying: {}", request_id, e),
                            unique_id,
                            x_request_id,
                        );
                        return Err(e);
                    }
                    
                    last_error = Some(e);
                    if attempt < webclient_config.max_retries {
                        StructuredLogger::log_warning(
                            &format!("Webhook attempt {} failed for request {}, retrying in {}s", 
                                attempt, request_id, webclient_config.retry_delay),
                            unique_id,
                            x_request_id,
                        );
                        sleep(Duration::from_secs(webclient_config.retry_delay)).await;
                    } else {
                        StructuredLogger::log_error(
                            &format!("All webhook attempts failed for request {}", request_id),
                            unique_id,
                            x_request_id,
                        );
                    }
                }
            }
        }
        
        Err(last_error.unwrap())
    }

    async fn make_webhook_request_with_context(
        &self,
        config: &PermataBankWebhookConfig,
        webhook_body: &str,
        request_id: &str,
        unique_id: Option<&str>,
        x_request_id: Option<&str>,
    ) -> Result<PermataWebhookResponse> {
        // Try webhook request with current token, handle 403 with relogin
        for retry_attempt in 0..2 { // Max 2 attempts: original + one retry after relogin
            // Get access token (will handle refresh if needed)
            let access_token = self.login_handler.get_token_with_context(unique_id, x_request_id).await?;
            
            // Generate timestamp for this request
            // let timestamp = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3f+07:00").to_string();
            let timestamp = &config.permata_timestamp;
            
            // Generate signature using permata_static_key:timestamp:webhook_body
            // First, compact the JSON to remove spaces and newlines
            let compacted_body = compact_json(webhook_body)?;
            let signature = generate_signature(
                &self.config.permata_bank_login.permata_static_key,
                &access_token,
                &timestamp,
                &compacted_body
            )?;

            StructuredLogger::log_info(
                &format!("Sending webhook to Permata Bank for request {} (attempt {})", request_id, retry_attempt + 1),
                unique_id,
                x_request_id,
                None,
            );
            
            let response = self.client
                .post(&config.callbackstatus_url)
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", access_token))
                .header("permata-signature", signature)
                .header("organizationname", &config.organizationname)
                .header("permata-timestamp", timestamp)
                .body(webhook_body.to_string())
                .send()
                .await?;

            let status = response.status();
            
            // Handle 403 authentication errors specifically
            if status == 403 {
                let body = response.text().await.unwrap_or_default();
                
                // Check if this is the specific OAuth2 authentication failure
                if body.contains("403") && body.contains("Failed Authentication OAuth2") {
                    StructuredLogger::log_warning(
                        &format!("Received 403 OAuth2 authentication error for request {}: {}", request_id, body),
                        unique_id,
                        x_request_id,
                    );
                    
                    if retry_attempt == 0 {
                        // Clear token cache and retry
                        StructuredLogger::log_info(
                            &format!("Clearing token cache and retrying login for request {}", request_id),
                            unique_id,
                            x_request_id,
                            None,
                        );
                        self.login_handler.clear_cache_with_context(unique_id, x_request_id);
                        continue; // Retry the request with new token
                    } else {
                        // Already retried once, return the error
                        StructuredLogger::log_error(
                            &format!("Authentication failed after relogin attempt for request {}: {}", request_id, body),
                            unique_id,
                            x_request_id,
                        );
                        return Err(crate::utils::error::AppError::authentication_failed(
                            format!("Failed Authentication OAuth2 after relogin: {}", body)
                        ));
                    }
                } else {
                    // Other 403 errors, don't retry
                    StructuredLogger::log_error(
                        &format!("Permata webhook failed with 403 (non-OAuth2) for request {}: {}", request_id, body),
                        unique_id,
                        x_request_id,
                    );
                    return Err(crate::utils::error::AppError::message_processing(
                        format!("Permata webhook failed: {} - {}", status, body)
                    ));
                }
            }
            
            // Handle other non-success status codes
            if !status.is_success() {
                let body = response.text().await.unwrap_or_default();
                StructuredLogger::log_error(
                    &format!("Permata webhook failed with status {} for request {}: {}", status, request_id, body),
                    unique_id,
                    x_request_id,
                );
                return Err(crate::utils::error::AppError::message_processing(
                    format!("Permata webhook failed: {} - {}", status, body)
                ));
            }

            // Success case - parse response
            let webhook_response: PermataWebhookResponse = response.json().await?;
            
            // Check if Permata Bank returned success
            if webhook_response.status_code != "00" {
                StructuredLogger::log_error(
                    &format!("Permata Bank returned error for request {}: {} - {}", 
                           request_id, webhook_response.status_code, webhook_response.status_desc),
                    unique_id,
                    x_request_id,
                );
                return Err(crate::utils::error::AppError::message_processing(
                    format!("Permata Bank error: {} - {}", webhook_response.status_code, webhook_response.status_desc)
                ));
            }

            StructuredLogger::log_info(
                &format!("Permata webhook success for request {}: {}", request_id, webhook_response.status_desc),
                unique_id,
                x_request_id,
                None,
            );
            return Ok(webhook_response);
        }
        
        // Should never reach here due to the loop logic, but just in case
        Err(crate::utils::error::AppError::message_processing(
            "Unexpected error in webhook retry logic".to_string()
        ))
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