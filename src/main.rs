use anyhow::Result;
use std::sync::Arc;
use tokio::signal;
use tracing::info;

use webhook_gateway::{
    config::AppConfig,
    services::WebhookProcessor,
    handlers::{WebhookServer, WebhookServerTrait},
    providers::StructuredLogger,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Check if we should run in test mode
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "--test" {
        return run_tests().await;
    }

    let config = AppConfig::load()?;
    
    StructuredLogger::init("info", Some(config.logger.clone()))?;
    
    info!("Starting Webhook Gateway Application");
    
    let webhook_processor = WebhookProcessor::new(config.clone())?;
    let webhook_server = WebhookServer::new(config.server.clone(), Arc::new(webhook_processor));

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

async fn run_tests() -> Result<()> {
    println!("=== Webhook Gateway Tests ===\n");
    
    // Note: Using built-in test functions (moved to tests directory)
    // For now, we'll do basic validation here
    
    // Test configuration loading
    match AppConfig::load() {
        Ok(config) => {
            println!("✓ Configuration loaded successfully");
            println!("  - Server host: {}", config.server.listen_host);
            println!("  - Server port: {}", config.server.listen_port);
            println!("  - Webhook path: {}", config.server.webhook_path);
            println!("✓ Configuration tests passed");
        }
        Err(e) => {
            println!("✗ Configuration tests failed: {}", e);
            return Err(e.into());
        }
    }
    
    // Test webhook processor
    let config_for_processor = AppConfig::load()?;
    let _processor = WebhookProcessor::new(config_for_processor)?;
    println!("✓ Webhook processor created successfully");
    println!("✓ Webhook processor tests passed");
    
    println!("\n=== All Tests Passed ===");
    Ok(())
}
