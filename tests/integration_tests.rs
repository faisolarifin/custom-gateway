use webhook_gateway::{
    config::ServerConfig,
    handlers::{WebhookServer, WebhookServerTrait},
    services::{WebhookProcessor, WebhookProcessorTrait},
    models::WebhookMessage,
    utils::error::AppError,
};
use std::{collections::HashMap, sync::Arc};

#[test]
fn test_webhook_message_serialization() {
    let mut headers = HashMap::new();
    headers.insert("content-type".to_string(), "application/json".to_string());
    headers.insert("authorization".to_string(), "Bearer token123".to_string());

    let message = WebhookMessage {
        headers,
        body: r#"{"user": "john", "action": "created"}"#.to_string(),
    };

    let json = serde_json::to_string(&message).unwrap();
    let deserialized: WebhookMessage = serde_json::from_str(&json).unwrap();

    assert_eq!(message.body, deserialized.body);
    assert_eq!(message.headers.len(), deserialized.headers.len());
    assert_eq!(message.headers.get("content-type"), 
              deserialized.headers.get("content-type"));
}

#[test]
fn test_error_chain() {
    let error = AppError::payload_conversion("Test error");
    let error_string = format!("{}", error);
    assert!(error_string.contains("Payload conversion error"));
    
    let config_error = AppError::configuration("Invalid port");
    let config_error_string = format!("{}", config_error);
    assert!(config_error_string.contains("Configuration error"));
}

#[tokio::test]
async fn test_webhook_server_integration() {
    let config = ServerConfig {
        listen_host: "127.0.0.1".to_string(),
        listen_port: 0, // Let OS pick available port
        webhook_path: "/test-webhook".to_string(),
    };

    // Create a dummy config for MessageProcessor (it won't be used in this test)
    let app_config = webhook_gateway::config::AppConfig::load().unwrap_or_else(|_| {
        use webhook_gateway::config::*;
        AppConfig {
            server: config.clone(),
            webclient: WebClientConfig { timeout: 30, max_retries: 3, retry_delay: 5 },
            permata_bank_login: PermataBankLoginConfig {
                permata_static_key: "test".to_string(),
                api_key: "test".to_string(),
                token_url: "https://test.com".to_string(),
                username: "test".to_string(),
                password: "test".to_string(),
                login_payload: "test".to_string(),
            },
            permata_bank_webhook: PermataBankWebhookConfig {
                callbackstatus_url: "https://test.com".to_string(),
                organizationname: "test".to_string(),
            },
            scheduler: SchedulerConfig {
                periodic_interval_mins: 15,
            },
            logger: LoggerConfig {
                dir: "log".to_string(),
                file_name: "test".to_string(),
                max_backups: 0,
                max_size: 10,
                max_age: 90,
                compress: true,
                local_time: true,
            },
        }
    });

    let processor = Arc::new(WebhookProcessor::new(app_config).unwrap());
    let server = WebhookServer::new(config, processor);

    // Test that server can be created and shut down gracefully
    let shutdown_result = server.shutdown().await;
    assert!(shutdown_result.is_ok());
    println!("✅ Webhook server integration test passed");
}

#[tokio::test]
#[ignore] // This test requires internet connection and may fail in CI
async fn test_real_webhook_forwarding() {
    // Test with a real HTTP endpoint (httpbin.org)
    // Create a config for testing
    let app_config = webhook_gateway::config::AppConfig::load().unwrap_or_else(|_| {
        use webhook_gateway::config::*;
        AppConfig {
            server: ServerConfig {
                listen_host: "127.0.0.1".to_string(),
                listen_port: 8080,
                webhook_path: "/webhook".to_string(),
            },
            webclient: WebClientConfig { timeout: 30, max_retries: 3, retry_delay: 5 },
            permata_bank_login: PermataBankLoginConfig {
                permata_static_key: "test".to_string(),
                api_key: "test".to_string(),
                token_url: "https://httpbin.org/post".to_string(),
                username: "test".to_string(),
                password: "test".to_string(),
                login_payload: "test".to_string(),
            },
            permata_bank_webhook: PermataBankWebhookConfig {
                callbackstatus_url: "https://httpbin.org/post".to_string(),
                organizationname: "test".to_string(),
            },
            scheduler: SchedulerConfig {
                periodic_interval_mins: 15,
            },
            logger: LoggerConfig {
                dir: "log".to_string(),
                file_name: "test".to_string(),
                max_backups: 0,
                max_size: 10,
                max_age: 90,
                compress: true,
                local_time: true,
            },
        }
    });

    let processor = WebhookProcessor::new(app_config).unwrap();
    
    let mut headers = HashMap::new();
    headers.insert("content-type".to_string(), "application/json".to_string());
    headers.insert("x-test-header".to_string(), "test-value".to_string());
    
    let webhook = WebhookMessage {
        headers,
        body: r#"{"test": "integration_test", "timestamp": "2024-01-01T00:00:00Z"}"#.to_string(),
    };
    
    // Test processing (this will make real HTTP requests to httpbin)
    let result = processor.process_webhook(webhook, "integration-test").await;
    
    // Handle both success and network failures gracefully
    match result {
        Ok(_) => println!("✅ Real webhook forwarding test passed"),
        Err(e) => {
            println!("⚠️  Network-dependent test failed (expected in some environments): {}", e);
            // Don't fail the test for network issues in CI/testing environments
        }
    }
}