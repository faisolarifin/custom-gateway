use std::time::Duration;

use reqwest::Client;
use tokio::time::sleep;

use crate::config::AppConfig;
use crate::services::{LoginHandler, TelegramAlertService};
use crate::providers::StructuredLogger;
use crate::utils::{error::Result, generate_signature};

#[derive(Debug, Clone)]
pub struct HttpWebhookResponse {
    pub status_code: u16,
    pub body: String,
}

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

    pub async fn send_webhook(&self, webhook_body: &str, request_id: &str) -> Result<HttpWebhookResponse> {
        self.send_webhook_with_context(webhook_body, request_id, Some(request_id), Some(request_id)).await
    }

    pub async fn send_webhook_with_context(&self, webhook_body: &str, request_id: &str, unique_id: Option<&str>, x_request_id: Option<&str>) -> Result<HttpWebhookResponse> {
        let webclient_config = &self.config.webclient;
        
        let mut last_error = None;
        
        for attempt in 1..=webclient_config.max_retries {
            match self.make_webhook_request(webhook_body, request_id, unique_id, x_request_id).await {
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

    async fn make_webhook_request(
        &self,
        webhook_body: &str,
        request_id: &str,
        unique_id: Option<&str>,
        x_request_id: Option<&str>,
    ) -> Result<HttpWebhookResponse> {
        // Get access token (will handle refresh if needed)
        let access_token = self.login_handler.get_token_with_context(unique_id, x_request_id).await?;
        
        // Generate timestamp for this request
        let timestamp = chrono::Utc::now().with_timezone(&chrono::FixedOffset::east_opt(7 * 3600)
                                .unwrap())
                                .format("%Y-%m-%dT%H:%M:%S%.3f+07:00")
                                .to_string();

        // Generate signature using permata_static_key:timestamp:webhook_body
        // First, compact the JSON to remove spaces and newlines
        let compacted_body = webhook_body.chars().filter(|c| !c.is_whitespace()).collect::<String>();
        let signature = generate_signature(
            &self.config.permata_bank_login.permata_static_key,
            &access_token,
            &timestamp,
            &compacted_body
        )?;

        StructuredLogger::log_info(
            &format!("Sending webhook to Permata Bank for request {}", request_id),
            unique_id,
            x_request_id,
            None,
        );
        
        let response = match self.client
            .post(&self.config.permata_bank_webhook.callbackstatus_url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", access_token))
            .header("permata-signature", signature)
            .header("organizationname", &self.config.permata_bank_webhook.organizationname)
            .header("permata-timestamp", timestamp)
            .body(webhook_body.to_string())
            .send()
            .await
        {
            Err(e) => {
                let error_message = "Request timeout/connection error for Permata Bank";
                
                StructuredLogger::log_error(
                    &format!("{}: {}", error_message, e),
                    unique_id,
                    x_request_id,
                );

                return Err(crate::utils::error::AppError::error(error_message));
            },
            Ok(resp) => resp,
        };

        let status_code = response.status().as_u16();
        let body = response.text().await.unwrap_or_default();
        
        // Log based on status code type
        if status_code >= 200 && status_code < 300 {
            StructuredLogger::log_info(
                &format!("Received HTTP {} from Permata Bank for request {}", status_code, request_id),
                unique_id,
                x_request_id,
                None,
            );
        } else {
            let error_message = format!("Received Error {} from Permata Bank for: {}", status_code, body);
            
            StructuredLogger::log_error(
                &error_message,
                unique_id,
                x_request_id,
            );
            
            // Send telegram alert for non-2xx status
            if let Ok(telegram_service) = TelegramAlertService::new(self.config.clone()) {
                telegram_service.send_error_alert(
                    format!("Received non-2xx HTTP {} from Permata Bank", status_code).as_str(),
                    x_request_id
                );
            }
        }
        
        Ok(HttpWebhookResponse {
            status_code,
            body,
        })
    }

    pub async fn shutdown(&self) {
        StructuredLogger::log_info(
            "Shutting down PermataCallbackStatusClient",
            None,
            None,
            None,
        );
        self.login_handler.shutdown().await;
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