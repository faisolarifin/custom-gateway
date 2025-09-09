use anyhow::Result;
use std::sync::Arc;
use tokio::signal;
use tracing::info;

use webhook_gateway::{
    config::AppConfig,
    services::{WebhookProcessor, WebhookProcessorTrait},
    handlers::{WebhookServer, WebhookServerTrait},
    providers::StructuredLogger,
};

#[tokio::main]
async fn main() -> Result<()> {
    let config = AppConfig::load()?;
    
    StructuredLogger::init("info", Some(config.logger.clone()))?;
    
    info!("Starting Webhook Gateway Application");
    
    let webhook_processor = WebhookProcessor::new(config.clone())?;
    let webhook_processor_arc: Arc<dyn WebhookProcessorTrait + Send + Sync> = Arc::new(webhook_processor.clone());
    let webhook_server = WebhookServer::new(config.server.clone(), webhook_processor_arc, config.clone());

    StructuredLogger::log_info(
        "Webhook Gateway Application started successfully",
        None,
        None,
        Some(serde_json::json!({
            "listen_address": format!("{}:{}", config.server.listen_host, config.server.listen_port),
            "webhook_path": config.server.webhook_path,
            "permata_callback_url": config.permata_bank_webhook.callbackstatus_url
        })),
    );

    let server_handle = tokio::spawn({
        let server = webhook_server.clone();
        async move {
            if let Err(e) = server.start().await {
                StructuredLogger::log_error(
                    &format!("Webhook server error: {}", e),
                    None,
                    None,
                );
            }
        }
    });

    match signal::ctrl_c().await {
        Ok(()) => {
            StructuredLogger::log_info(
                "Shutdown signal received, initiating graceful shutdown",
                None,
                None,
                None,
            );
        }
        Err(e) => {
            StructuredLogger::log_error(
                &format!("Failed to listen for shutdown signal: {}", e),
                None,
                None,
            );
        }
    }

    // Graceful shutdown sequence
    StructuredLogger::log_info(
        "Starting graceful shutdown sequence",
        None,
        None,
        None,
    );

    // Stop the webhook server
    if let Err(e) = webhook_server.shutdown().await {
        StructuredLogger::log_error(
            &format!("Error during webhook server shutdown: {}", e),
            None,
            None,
        );
    }

    // Stop the webhook processor (including token scheduler)
    webhook_processor.shutdown().await;

    // Cancel the server task
    server_handle.abort();

    StructuredLogger::log_info(
        "Webhook Gateway Application stopped",
        None,
        None,
        None,
    );

    Ok(())
}
