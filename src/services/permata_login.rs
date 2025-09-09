use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use reqwest::Client;
use tokio::time::sleep;

use crate::config::{AppConfig, PermataBankLoginConfig};
use crate::models::TokenResponse;
use crate::providers::StructuredLogger;
use crate::utils::{error::Result, generate_signature};
use crate::services::{TokenScheduler, TelegramAlertService};

#[derive(Clone)]
pub struct LoginHandler {
    client: Client,
    config: AppConfig,
    token_cache: Arc<Mutex<HashMap<String, CachedToken>>>,
    token_scheduler: TokenScheduler,
}

#[derive(Debug, Clone)]
struct CachedToken {
    token: String,
    expires_at: Instant,
}

impl LoginHandler {
    pub fn new(config: AppConfig) -> Result<Self> {
        let timeout = Duration::from_secs(config.webclient.timeout);
        let client = Client::builder()
            .timeout(timeout)
            .build()?;
        
        let scheduler = TokenScheduler::with_config(config.token_scheduler.clone());

        let handler = Self {
            client,
            config,
            token_cache: Arc::new(Mutex::new(HashMap::new())),
            token_scheduler: scheduler,
        };
        
        // Start periodic scheduler immediately
        handler.start_periodic_token_refresh();
        
        Ok(handler)
    }

    pub async fn get_token(&self) -> Result<String> {
        self.get_token_with_context(None, None).await
    }

    pub async fn get_token_with_context(&self, unique_id: Option<&str>, request_id: Option<&str>) -> Result<String> {
        let cache_key = "permata_bank_token";
        
        // Check cache first
        {
            let cache = self.token_cache.lock().unwrap();
            if let Some(cached_token) = cache.get(cache_key) {
                if cached_token.expires_at > Instant::now() {
                    StructuredLogger::log_info(
                        "Using cached token",
                        unique_id,
                        request_id,
                        None,
                    );
                    return Ok(cached_token.token.clone());
                }
            }
        }

        // Token not in cache or expired, fetch new one
        StructuredLogger::log_info(
            "Fetching new token from API",
            unique_id,
            request_id,
            None,
        );
        let token_response = self.login_with_context(unique_id, request_id).await?;
        
        // Cache the token (subtract 5 minutes from expires_in for safety)
        let expires_at = Instant::now() + Duration::from_secs(token_response.expires_in.saturating_sub(300));
        let cached_token = CachedToken {
            token: token_response.access_token.clone(),
            expires_at,
        };

        {
            let mut cache = self.token_cache.lock().unwrap();
            cache.insert(cache_key.to_string(), cached_token);
        }

        // Periodic scheduler sudah berjalan, tidak perlu start manual scheduler

        Ok(token_response.access_token)
    }

    fn start_periodic_token_refresh(&self) {
        let cache = Arc::clone(&self.token_cache);
        let handler_clone = self.clone();

        // Start periodic scheduler yang berjalan setiap 15 menit (atau sesuai config)
        self.token_scheduler.start_scheduler(move || {
            let cache_clone = Arc::clone(&cache);
            let handler_clone = handler_clone.clone();
            
            async move {
                StructuredLogger::log_info(
                    "Periodic token refresh triggered - clearing cache and fetching new token",
                    None,
                    None,
                    None,
                );
                
                // Clear cache dan fetch token baru
                {
                    let mut cache_guard = cache_clone.lock().unwrap();
                    cache_guard.clear();
                }
                
                // Trigger token refresh dengan call get_token
                handler_clone.get_token_with_context(None, Some("scheduler")).await
                    .map(|_| ())
            }
        });
    }

    async fn login_with_context(&self, unique_id: Option<&str>, request_id: Option<&str>) -> Result<TokenResponse> {
        let login_config = &self.config.permata_bank_login;
        let webclient_config = &self.config.webclient;
        
        let mut last_error = None;
        
        for attempt in 1..=webclient_config.max_retries {
            match self.make_login_request_with_context(login_config, unique_id, request_id).await {
                Ok(response) => {
                    StructuredLogger::log_info(
                        &format!("Login successful on attempt {}", attempt),
                        unique_id,
                        request_id,
                        None,
                    );
                    return Ok(response);
                }
                Err(e) => {
                    last_error = Some(e);
                    if attempt < webclient_config.max_retries {
                        StructuredLogger::log_warning(
                            &format!("Login attempt {} failed, retrying in {}s", attempt, webclient_config.retry_delay),
                            unique_id,
                            request_id,
                        );
                        sleep(Duration::from_secs(webclient_config.retry_delay)).await;
                    } else {
                        StructuredLogger::log_error(
                            "All login attempts failed",
                            unique_id,
                            request_id,
                        );
        
                    }
                }
            }
        }
        
        Err(last_error.unwrap())
    }

    async fn make_login_request_with_context(&self, config: &PermataBankLoginConfig, unique_id: Option<&str>, request_id: Option<&str>) -> Result<TokenResponse> {
        // Generate timestamp for this request
        let timestamp = chrono::Utc::now().with_timezone(&chrono::FixedOffset::east_opt(7 * 3600)
                                .unwrap())
                                .format("%Y-%m-%dT%H:%M:%S%.3f+07:00")
                                .to_string();
        
        // Create Basic Auth header (base64 encode username:password)
        let auth_string = format!("{}:{}", config.username, config.password);
        let auth_header = format!("Basic {}", base64::Engine::encode(&base64::engine::general_purpose::STANDARD, auth_string.as_bytes()));
        
        // Generate signature using key:timestamp:data format
        let signature = generate_signature(
            &config.permata_static_key,
            &config.api_key,
            &timestamp,
            &config.login_payload
        )?;

        let response = self.client
            .post(&config.token_url)
            .header("Authorization", auth_header)
            .header("OAUTH-Signature", signature)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("OAUTH-Timestamp", timestamp)
            .header("API-Key", &config.api_key)
            .body(config.login_payload.clone())
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            let error_message = format!("Login request failed with status {}: {}", status, body);
            
            StructuredLogger::log_error(
                &error_message,
                unique_id,
                request_id,
            );
            
            // Send telegram alert for individual login request failures
            if let Ok(telegram_service) = TelegramAlertService::new(self.config.clone()) {
                telegram_service.send_error_alert(
                    &error_message,
                    request_id
                );
            }
            
            return Err(crate::utils::error::AppError::authentication_failed(
                format!("Login failed: {} - {}", status, body)
            ));
        }

        let token_response: TokenResponse = response.json().await?;
        StructuredLogger::log_info(
            &format!("Successfully obtained token, expires in {} seconds", token_response.expires_in),
            unique_id,
            request_id,
            None,
        );
        
        Ok(token_response)
    }

    pub fn clear_cache(&self) {
        self.clear_cache_with_context(None, None);
    }

    pub fn clear_cache_with_context(&self, unique_id: Option<&str>, request_id: Option<&str>) {
        let mut cache = self.token_cache.lock().unwrap();
        cache.clear();
        StructuredLogger::log_info(
            "Token cache cleared",
            unique_id,
            request_id,
            None,
        );
        
        // Stop scheduler saat clear cache manual
        self.token_scheduler.stop_scheduler();
    }

    pub fn stop_scheduler(&self) {
        self.token_scheduler.stop_scheduler();
    }

    // Method untuk check status scheduler
    pub fn is_scheduler_active(&self) -> bool {
        self.token_scheduler.is_scheduler_active()
    }

    pub fn get_scheduler_info(&self) -> Option<String> {
        self.token_scheduler.get_scheduler_info()
    }

    // Method untuk shutdown gracefully
    pub async fn shutdown(&self) {
        StructuredLogger::log_info(
            "Shutting down LoginHandler and stopping scheduler",
            None,
            None,
            None,
        );
        self.token_scheduler.shutdown();
    }
}