use async_trait::async_trait;
use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{error, info};
use uuid::Uuid;

use crate::config::ServerConfig;
use crate::services::{WebhookProcessorTrait, TelegramAlertService};
use crate::utils::error::{AppError, Result};
use crate::utils::json::{is_dr_payload, is_inbound_flow_payload};
use crate::providers::logging::StructuredLogger;

#[async_trait]
pub trait WebhookServerTrait {
    async fn start(&self) -> Result<()>;
    async fn shutdown(&self) -> Result<()>;
}

#[derive(Clone)]
pub struct WebhookServer {
    config: ServerConfig,
    processor: Arc<dyn WebhookProcessorTrait + Send + Sync>,
    app_config: crate::config::AppConfig,
}

impl WebhookServer {
    pub fn new(config: ServerConfig, processor: Arc<dyn WebhookProcessorTrait + Send + Sync>, app_config: crate::config::AppConfig) -> Self {
        Self { config, processor, app_config }
    }

    async fn handle_webhook(
        &self,
        req: Request<hyper::body::Incoming>,
    ) -> std::result::Result<Response<Full<Bytes>>, Infallible> {
        let request_id = format!("req-{}", Uuid::new_v4());

        StructuredLogger::log_info(
            "Received webhook request",
            Some(&request_id),
            Some(&request_id),
            Some(serde_json::json!({
                "method": req.method().as_str(),
                "uri": req.uri().to_string(),
                "headers": req.headers().len()
            })),
        );

        let response = match (req.method(), req.uri().path()) {
            (&Method::POST, path) if path == self.config.webhook_path => {
                self.process_webhook(req, &request_id).await
            }
            (&Method::GET, path) if path == self.config.webhook_path => {
                self.handle_health_check(&request_id).await
            }
            _ => {
                StructuredLogger::log_info(
                    "Webhook path not found",
                    Some(&request_id),
                    Some(&request_id),
                    None,
                );
                Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Full::new(Bytes::from("Not Found")))
                    .unwrap())
            }
        };

        response
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

    async fn handle_health_check(
        &self,
        request_id: &str,
    ) -> std::result::Result<Response<Full<Bytes>>, Infallible> {
        StructuredLogger::log_info(
            "Health check request",
            Some(request_id),
            Some(request_id),
            None,
        );

        let health_response = serde_json::json!({
            "status": "success",
            "message": "Application is healthy"
        });

        Ok(Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "application/json")
            .body(Full::new(Bytes::from(health_response.to_string())))
            .unwrap())
    }

    async fn process_webhook(
        &self,
        req: Request<hyper::body::Incoming>,
        request_id: &str,
    ) -> std::result::Result<Response<Full<Bytes>>, Infallible> {
        // Extract headers for forwarding
        let headers = req.headers().clone();
        
        // Read the body
        let body_bytes = match req.collect().await {
            Ok(body) => body.to_bytes(),
            Err(e) => {
                StructuredLogger::log_error(
                    &format!("Failed to read request body: {}", e),
                    Some(request_id),
                    Some(request_id),
                );
                return Ok(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(Full::new(Bytes::from("Bad Request")))
                    .unwrap());
            }
        };

        // Parse JSON payload to determine if it's DR or Inbound Flow
        let body_str = String::from_utf8_lossy(&body_bytes);
        
        // Check if payload should be processed
        if !self.should_process_payload(&body_str, request_id) {
            StructuredLogger::log_info(
                "Ignore send payload to client",
                Some(request_id),
                Some(request_id),
                None,
            );
            return Ok(Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "application/json")
                .body(Full::new(Bytes::from(r#"{"StatusCode":"00","StatusDesc":"Success"}"#)))
                .unwrap());
        }

        // Create webhook message for processing
        let webhook_data = crate::models::WebhookMessage {
            headers: headers.iter()
                .map(|(k, v)| (k.as_str().to_string(), v.to_str().unwrap_or("").to_string()))
                .collect(),
            body: body_str.to_string(),
        };

        // Process the webhook
        match self.processor.process_webhook(webhook_data, request_id).await {
            Ok(webhook_response) => {
                // Langsung gunakan HTTP status dan body dari Permata Bank
                let http_status = StatusCode::from_u16(webhook_response.http_status)
                    .unwrap_or(StatusCode::BAD_GATEWAY);
                
                StructuredLogger::log_info(
                    &format!("Webhook processed with HTTP status {}", webhook_response.http_status),
                    Some(request_id),
                    Some(request_id),
                    None,
                );

                Ok(Response::builder()
                    .status(http_status)
                    .header("content-type", "application/json")
                    .body(Full::new(Bytes::from(webhook_response.body)))
                    .unwrap())
            }
            Err(e) => {
                StructuredLogger::log_error(
                    &format!("Failed to process webhook: {}", e),
                    Some(request_id),
                    Some(request_id),
                );

                let json_body = format!(r#"{{"StatusCode":"06","StatusDesc":"{}"}}"#, e);
                Ok(Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header("content-type", "application/json")
                    .body(Full::new(Bytes::from(json_body)))
                    .unwrap())
            }
        }
    }
}

#[async_trait]
impl WebhookServerTrait for WebhookServer {
    async fn start(&self) -> Result<()> {
        let addr: SocketAddr = format!("{}:{}", self.config.listen_host, self.config.listen_port)
            .parse()
            .map_err(|e| AppError::configuration(format!("Invalid server address: {}", e)))?;

        let listener = TcpListener::bind(addr)
            .await
            .map_err(|e| AppError::configuration(format!("Failed to bind to address {}: {}", addr, e)))?;

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

        loop {
            let (stream, peer_addr) = match listener.accept().await {
                Ok(conn) => conn,
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                    continue;
                }
            };

            let io = TokioIo::new(stream);
            let server_clone = self.clone();

            tokio::task::spawn(async move {
                let service = service_fn(move |req| {
                    let server = server_clone.clone();
                    async move { server.handle_webhook(req).await }
                });

                if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                    error!("Error serving connection from {}: {}", peer_addr, err);
                }
            });
        }
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