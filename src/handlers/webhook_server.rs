use async_trait::async_trait;
use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde_json::Value;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::signal;
use tracing::info;
use uuid::Uuid;

use crate::config::ServerConfig;
use crate::services::{WebhookProcessorTrait, TelegramAlertService};
use crate::utils::error::{AppError, Result};
use crate::utils::request_id::extract_request_id;
use crate::utils::json::{is_dr_payload, is_inbound_flow_payload};
use crate::providers::logging::StructuredLogger;

#[async_trait]
pub trait WebhookServerTrait {
    async fn start(&self) -> Result<()>;
    async fn shutdown(&self) -> Result<()>;
}

#[derive(Clone)]
pub struct AppState {
    pub processor: Arc<dyn WebhookProcessorTrait + Send + Sync>,
    pub app_config: crate::config::AppConfig,
    pub server_config: ServerConfig,
}

#[derive(Clone)]
pub struct WebhookServer {
    config: ServerConfig,
    processor: Arc<dyn WebhookProcessorTrait + Send + Sync>,
    app_config: crate::config::AppConfig,
}

impl WebhookServer {
    pub fn new(config: ServerConfig, processor: Arc<dyn WebhookProcessorTrait + Send + Sync>, app_config: crate::config::AppConfig) -> Self {
        Self { 
            config, 
            processor, 
            app_config,
        }
    }

    fn create_router(&self) -> Router {
        let app_state = AppState {
            processor: self.processor.clone(),
            app_config: self.app_config.clone(),
            server_config: self.config.clone(),
        };

        Router::new()
            .route(&self.config.webhook_path, post(webhook_handler))
            .route(&self.config.webhook_path, get(health_check_handler))
            .with_state(app_state)
    }

    fn should_process_payload(&self, body: &str, request_id: &str) -> bool {
        match serde_json::from_str::<serde_json::Value>(body) {
            Ok(json) => {              
                 // Check for DR (Delivery Receipt) payload
                if is_dr_payload(&json) {
                    StructuredLogger::log_info(
                        "Detected DR payload",
                        Some(request_id),
                        Some(request_id),
                        None,
                    );
                    return true;
                }
                  
                // Check for Inbound Flow payload
                if is_inbound_flow_payload(&json) {
                    StructuredLogger::log_info(
                        "Detected Inbound Flow payload",
                        Some(request_id),
                        Some(request_id),
                        None,
                    );
                    return true;
                }
                
                StructuredLogger::log_info(
                    "Payload does not match DR or Inbound Flow criteria",
                    Some(request_id),
                    Some(request_id),
                    None,
                );
                false
            }
            Err(e) => {
                let error_message = format!("Failed to parse JSON payload: {}", e);
                
                StructuredLogger::log_error(
                    &error_message,
                    Some(request_id),
                    Some(request_id),
                );
                
                // Send telegram alert for JSON parse failure
                if let Ok(telegram_service) = TelegramAlertService::new(self.app_config.clone()) {
                    telegram_service.send_error_alert(
                        "Failed to parse JSON payload",
                        Some(request_id)
                    );
                }
                false
            }
        }
    }
}

// Axum handler functions
pub async fn webhook_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    request: Request,
) -> impl IntoResponse {
    let request_id = format!("req-{}", Uuid::new_v4());

    StructuredLogger::log_info(
        "Received webhook request",
        Some(&request_id),
        Some(&request_id),
        Some(serde_json::json!({
            "method": "POST",
            "uri": request.uri().to_string(),
            "headers": headers.len()
        })),
    );

    // Extract the body
    let body = match axum::body::to_bytes(request.into_body(), usize::MAX).await {
        Ok(bytes) => bytes,
        Err(e) => {
            StructuredLogger::log_error(
                &format!("Failed to read request body: {}", e),
                Some(&request_id),
                Some(&request_id),
            );
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "StatusCode": "06",
                    "StatusDesc": "Bad Request"
                }))
            );
        }
    };

    let body_str = String::from_utf8_lossy(&body);
    let extracted_request_id = extract_request_id(&body_str);

    // Check if payload should be processed
    let server = WebhookServer {
        config: state.server_config.clone(),
        processor: state.processor.clone(),
        app_config: state.app_config.clone(),
    };

    if !server.should_process_payload(&body_str, &extracted_request_id) {
        StructuredLogger::log_info(
            "Ignore send payload to client",
            Some(&extracted_request_id),
            Some(&extracted_request_id),
            None,
        );
        return (
            StatusCode::OK,
            Json(serde_json::json!({
                "StatusCode": "00",
                "StatusDesc": "Success"
            }))
        );
    }

    // Create webhook message for processing
    let webhook_data = crate::models::WebhookMessage {
        headers: headers.iter()
            .map(|(k, v)| (k.as_str().to_string(), v.to_str().unwrap_or("").to_string()))
            .collect(),
        body: body_str.to_string(),
    };

    // Process the webhook
    match state.processor.process_webhook(webhook_data, &extracted_request_id).await {
        Ok(webhook_response) => {
            let http_status = StatusCode::from_u16(webhook_response.http_status)
                .unwrap_or(StatusCode::BAD_GATEWAY);
            
            StructuredLogger::log_info(
                &format!("Webhook processed with HTTP status {}", webhook_response.http_status),
                Some(&extracted_request_id),
                Some(&extracted_request_id),
                None,
            );

            // Parse the response body as JSON if possible
            let response_json: Value = serde_json::from_str(&webhook_response.body)
                .unwrap_or_else(|_| serde_json::json!({
                    "StatusCode": "06",
                    "StatusDesc": webhook_response.body
                }));

            (http_status, Json(response_json))
        }
        Err(e) => {
            StructuredLogger::log_error(
                &format!("Failed to process webhook: {}", e),
                Some(&extracted_request_id),
                Some(&extracted_request_id),
            );

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "StatusCode": "06",
                    "StatusDesc": e.to_string()
                }))
            )
        }
    }
}

pub async fn health_check_handler(
    State(_state): State<AppState>,
) -> impl IntoResponse {
    let request_id = format!("req-{}", Uuid::new_v4());
    
    StructuredLogger::log_info(
        "Health check request",
        Some(&request_id),
        Some(&request_id),
        None,
    );

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "success",
            "message": "Application is healthy"
        }))
    )
}

#[async_trait]
impl WebhookServerTrait for WebhookServer {
    async fn start(&self) -> Result<()> {
        let addr: SocketAddr = format!("{}:{}", self.config.listen_host, self.config.listen_port)
            .parse()
            .map_err(|e| AppError::configuration(format!("Invalid server address: {}", e)))?;

        let app = self.create_router();

        info!("Webhook server listening on {}", addr);
        StructuredLogger::log_info(
            "Webhook server started",
            None,
            None,
            Some(serde_json::json!({
                "address": addr.to_string(),
                "webhook_path": self.config.webhook_path
            })),
        );

        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|e| AppError::configuration(format!("Failed to bind to address {}: {}", addr, e)))?;

        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal())
            .await
            .map_err(|e| AppError::error(format!("Server error: {}", e)))?;

        Ok(())
    }

    async fn shutdown(&self) -> Result<()> {
        StructuredLogger::log_info(
            "Webhook server shutting down",
            None,
            None,
            None,
        );
        Ok(())
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    StructuredLogger::log_info(
        "Signal received, starting graceful shutdown",
        None,
        None,
        None,
    );
}