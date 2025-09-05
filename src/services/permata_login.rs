use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use reqwest::Client;
use tokio::time::sleep;

use crate::config::{AppConfig, PermataBankLoginConfig};
use crate::models::TokenResponse;
use crate::providers::StructuredLogger;
use crate::utils::{error::Result, generate_signature};

#[derive(Clone)]
pub struct LoginHandler {
    client: Client,
    config: AppConfig,
    token_cache: Arc<Mutex<HashMap<String, CachedToken>>>,
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

        Ok(Self {
            client,
            config,
            token_cache: Arc::new(Mutex::new(HashMap::new())),
        })
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

        Ok(token_response.access_token)
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
        // let timestamp = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3f+07:00").to_string();
        let timestamp = &config.oauth_timestamp;
        
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
            StructuredLogger::log_error(
                &format!("Login request failed with status {}: {}", status, body),
                unique_id,
                request_id,
            );
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
    }
}