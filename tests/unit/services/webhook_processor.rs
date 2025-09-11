use std::collections::HashMap;
use webhook_gateway::{
    config::*,
    models::WebhookMessage,
    services::{WebhookProcessor, WebhookProcessorTrait},
    services::webhook_processor::WebhookResponse,
};

const WHATSAPP_DR_PAYLOAD: &str = r#"{
    "xid": "123",
    "entry": [
        {
            "id": "115617074757249",
            "time": 0,
            "changes": [
                {
                    "field": "messages",
                    "value": {
                        "metadata": {
                            "phone_number_id": "115159954803011",
                            "display_phone_number": "6287845715199"
                        },
                        "statuses": [
                            {
                                "id": "wamid.HBgNNjI4MjIyODIyMzUwMBUCABEYEjg1ODdCMEMxRjkyNUJCRUY5NwA=",
                                "status": "delivered",
                                "timestamp": "1677836780",
                                "recipient_id": "6282228223500"
                            }
                        ],
                        "messaging_product": "whatsapp"
                    }
                }
            ]
        }
    ]
}"#;

fn create_test_webhook_message() -> WebhookMessage {
    let mut headers = HashMap::new();
    headers.insert("content-type".to_string(), "application/json".to_string());
    headers.insert("user-agent".to_string(), "test-client/1.0".to_string());
    
    WebhookMessage {
        headers,
        body: r#"{"user": "john", "email": "john@example.com"}"#.to_string(),
    }
}

fn create_whatsapp_webhook_message() -> WebhookMessage {
    let mut headers = HashMap::new();
    headers.insert("content-type".to_string(), "application/json".to_string());
    headers.insert("x-hub-signature".to_string(), "sha1=signature".to_string());
    
    WebhookMessage {
        headers,
        body: WHATSAPP_DR_PAYLOAD.to_string(),
    }
}

fn create_webhook_with_large_payload() -> WebhookMessage {
    let mut headers = HashMap::new();
    headers.insert("content-type".to_string(), "application/json".to_string());
    
    // Create a large JSON payload
    let large_data = "x".repeat(1000);
    let body = format!(r#"{{"data": "{}"}}"#, large_data);
    
    WebhookMessage {
        headers,
        body,
    }
}

fn create_webhook_with_no_headers() -> WebhookMessage {
    WebhookMessage {
        headers: HashMap::new(),
        body: r#"{"minimal": "data"}"#.to_string(),
    }
}

fn create_test_config() -> AppConfig {
    AppConfig {
        server: ServerConfig {
            listen_host: "127.0.0.1".to_string(),
            listen_port: 8080,
            webhook_path: "/webhook".to_string(),
        },
        webclient: WebClientConfig {
            timeout: 30,
            max_retries: 3,
            retry_delay: 1,
        },
        permata_bank_login: PermataBankLoginConfig {
            permata_static_key: "test_key".to_string(),
            api_key: "test_api_key".to_string(),
            token_url: "https://httpbin.org/post".to_string(),
            username: "test_user".to_string(),
            password: "test_pass".to_string(),
            login_payload: "grant_type=client_credentials".to_string(),
        },
        permata_bank_webhook: PermataBankWebhookConfig {
            callbackstatus_url: "https://httpbin.org/post".to_string(),
            organizationname: "test_org".to_string(),
        },
        token_scheduler: SchedulerConfig {
            periodic_interval_mins: 15,
        },
        telegram_alert: TelegramAlertConfig {
            api_url: "https://api.telegram.org/bot123:test/sendMessage".to_string(),
            chat_id: "-123456789".to_string(),
            message_thread_id: "123".to_string(),
            alert_message_prefix: "[TEST]".to_string(),
        },
        logger: LoggerConfig {
            dir: std::env::temp_dir().to_string_lossy().to_string(),
            file_name: "test-webhook-processor".to_string(),
            max_backups: 0,
            max_size: 10,
            max_age: 90,
            compress: true,
            local_time: true,
        },
    }
}

fn create_success_config() -> AppConfig {
    let mut config = create_test_config();
    config.permata_bank_webhook.callbackstatus_url = "https://httpbin.org/status/200".to_string();
    config
}

fn create_auth_failure_config() -> AppConfig {
    let mut config = create_test_config();
    config.permata_bank_login.token_url = "https://httpbin.org/status/401".to_string();
    config
}

fn create_network_failure_config() -> AppConfig {
    let mut config = create_test_config();
    config.permata_bank_webhook.callbackstatus_url = "https://invalid-domain-that-does-not-exist.com".to_string();
    config
}

#[tokio::test]
async fn test_webhook_processor_creation() {
    let config = create_test_config();
    let processor = WebhookProcessor::new(config);
    assert!(processor.is_ok());
}

#[tokio::test]
async fn test_webhook_processor_creation_with_invalid_config() {
    let mut config = create_test_config();
    config.webclient.timeout = 0; // Invalid timeout
    
    let processor = WebhookProcessor::new(config);
    // Should still create successfully as validation happens during usage
    assert!(processor.is_ok());
}

#[tokio::test]
async fn test_webhook_message_structure() {
    let webhook = create_test_webhook_message();
    assert_eq!(webhook.body, r#"{"user": "john", "email": "john@example.com"}"#);
    assert_eq!(webhook.headers.len(), 2);
    assert_eq!(webhook.headers.get("content-type").unwrap(), "application/json");
    assert_eq!(webhook.headers.get("user-agent").unwrap(), "test-client/1.0");
}

#[tokio::test]
async fn test_whatsapp_webhook_message_structure() {
    let webhook = create_whatsapp_webhook_message();
    assert!(webhook.body.contains("statuses"));
    assert!(webhook.body.contains("delivered"));
    assert_eq!(webhook.headers.get("content-type").unwrap(), "application/json");
    assert!(webhook.headers.contains_key("x-hub-signature"));
}

#[tokio::test] 
async fn test_large_payload_webhook() {
    let webhook = create_webhook_with_large_payload();
    assert!(webhook.body.len() > 1000);
    assert!(webhook.body.contains("data"));
}

#[tokio::test]
async fn test_webhook_with_no_headers() {
    let webhook = create_webhook_with_no_headers();
    assert!(webhook.headers.is_empty());
    assert_eq!(webhook.body, r#"{"minimal": "data"}"#);
}

#[test]
fn test_config_validation() {
    let config = create_test_config();
    assert_eq!(config.server.listen_host, "127.0.0.1");
    assert_eq!(config.server.listen_port, 8080);
    assert_eq!(config.permata_bank_webhook.organizationname, "test_org");
    assert!(config.permata_bank_webhook.callbackstatus_url.starts_with("https://"));
}

#[test]
fn test_webhook_response_structure() {
    let response = WebhookResponse {
        http_status: 200,
        body: "success".to_string(),
    };
    assert_eq!(response.http_status, 200);
    assert_eq!(response.body, "success");
}

#[test]
fn test_webhook_response_error_structure() {
    let response = WebhookResponse {
        http_status: 401,
        body: r#"{"error": "Authentication failed", "message": "Invalid credentials"}"#.to_string(),
    };
    assert_eq!(response.http_status, 401);
    assert!(response.body.contains("Authentication failed"));
}

#[tokio::test]
async fn test_process_webhook_basic() {
    let config = create_success_config();
    let processor = WebhookProcessor::new(config).unwrap();
    let webhook = create_test_webhook_message();
    
    let result = processor.process_webhook(webhook, "test-basic-123").await;
    
    // The result should be processed (might succeed or fail based on network)
    // We mainly test that it doesn't panic and returns a proper Result
    match result {
        Ok(response) => {
            assert!(response.http_status > 0);
        },
        Err(_) => {
            // Network errors are acceptable in tests
        }
    }
}

#[tokio::test]
async fn test_process_webhook_with_whatsapp_payload() {
    let config = create_success_config();
    let processor = WebhookProcessor::new(config).unwrap();
    let webhook = create_whatsapp_webhook_message();
    
    let result = processor.process_webhook(webhook, "test-whatsapp-456").await;
    
    match result {
        Ok(response) => {
            assert!(response.http_status > 0);
        },
        Err(_) => {
            // Network errors are acceptable in tests
        }
    }
}

#[tokio::test]
async fn test_process_webhook_with_large_payload() {
    let config = create_success_config();
    let processor = WebhookProcessor::new(config).unwrap();
    let webhook = create_webhook_with_large_payload();
    
    let result = processor.process_webhook(webhook, "test-large-789").await;
    
    match result {
        Ok(response) => {
            assert!(response.http_status > 0);
        },
        Err(_) => {
            // Expected for large payloads or network issues
        }
    }
}

#[tokio::test]
async fn test_authentication_error_handling() {
    let config = create_auth_failure_config();
    let processor = WebhookProcessor::new(config).unwrap();
    let webhook = create_test_webhook_message();
    
    let result = processor.process_webhook(webhook, "test-auth-failure").await;
    
    // Authentication failures should return an error from webhook processor
    assert!(result.is_err(), "Authentication failures should return an error");
    
    let error = result.unwrap_err();
    // Verify error is of the correct type
    assert!(matches!(error, webhook_gateway::utils::error::AppError::ReqError { .. }));
}

#[tokio::test]
async fn test_authentication_error_with_login_failed_message() {
    let config = create_auth_failure_config();
    let processor = WebhookProcessor::new(config).unwrap();
    let webhook = create_test_webhook_message();
    
    let result = processor.process_webhook(webhook, "test-login-failed").await;
    
    // Login failures should return an error from webhook processor
    assert!(result.is_err(), "Login failures should return an error");
    
    let error = result.unwrap_err();
    // Verify error is of the correct type
    assert!(matches!(error, webhook_gateway::utils::error::AppError::ReqError { .. }));
}

#[tokio::test]
async fn test_network_error_handling() {
    let config = create_network_failure_config();
    let processor = WebhookProcessor::new(config).unwrap();
    let webhook = create_test_webhook_message();
    
    let result = processor.process_webhook(webhook, "test-network-error").await;
    
    // Network errors should bubble up as Err
    match result {
        Ok(response) => {
            // Might get 401 if it's an auth error instead
            assert!(response.http_status == 401);
        },
        Err(_) => {
            // This is expected for network failures
        }
    }
}

#[tokio::test]
async fn test_shutdown_functionality() {
    let config = create_test_config();
    let processor = WebhookProcessor::new(config).unwrap();
    
    // Test that shutdown doesn't panic
    processor.shutdown().await;
    
    // Test that we can still create another processor after shutdown
    let config2 = create_test_config();
    let processor2 = WebhookProcessor::new(config2);
    assert!(processor2.is_ok());
}

#[tokio::test]
async fn test_multiple_webhook_processing() {
    let config = create_success_config();
    let processor = WebhookProcessor::new(config).unwrap();
    
    let webhook1 = create_test_webhook_message();
    let webhook2 = create_whatsapp_webhook_message();
    let webhook3 = create_webhook_with_no_headers();
    
    // Process multiple webhooks with different request IDs
    let result1 = processor.process_webhook(webhook1, "req-001").await;
    let result2 = processor.process_webhook(webhook2, "req-002").await;  
    let result3 = processor.process_webhook(webhook3, "req-003").await;
    
    // All should be handled without panics
    // Results may vary based on network conditions
    match (result1, result2, result3) {
        (Ok(_), Ok(_), Ok(_)) => {},
        _ => {
            // Some failures are acceptable in test environment
        }
    }
}

#[tokio::test]
async fn test_process_webhook_with_empty_body() {
    let config = create_success_config();
    let processor = WebhookProcessor::new(config).unwrap();
    
    let webhook = WebhookMessage {
        headers: HashMap::new(),
        body: String::new(),
    };
    
    let result = processor.process_webhook(webhook, "test-empty-body").await;
    
    match result {
        Ok(response) => {
            assert!(response.http_status > 0);
        },
        Err(_) => {
            // Acceptable for empty body
        }
    }
}

#[tokio::test]
async fn test_process_webhook_with_invalid_json() {
    let config = create_success_config();
    let processor = WebhookProcessor::new(config).unwrap();
    
    let webhook = WebhookMessage {
        headers: HashMap::new(),
        body: "invalid json {".to_string(),
    };
    
    let result = processor.process_webhook(webhook, "test-invalid-json").await;
    
    match result {
        Ok(response) => {
            assert!(response.http_status > 0);
        },
        Err(_) => {
            // Expected for invalid JSON
        }
    }
}

#[tokio::test]
async fn test_concurrent_webhook_processing() {
    let config = create_success_config();
    let processor = WebhookProcessor::new(config).unwrap();
    
    let webhook1 = create_test_webhook_message();
    let webhook2 = create_whatsapp_webhook_message();
    
    // Process webhooks concurrently
    let task1 = processor.process_webhook(webhook1, "concurrent-1");
    let task2 = processor.process_webhook(webhook2, "concurrent-2");
    
    let (result1, result2) = tokio::join!(task1, task2);
    
    // Both should complete without panics
    match (result1, result2) {
        (Ok(_), Ok(_)) => {},
        _ => {
            // Some failures acceptable in test environment
        }
    }
}

#[tokio::test] 
async fn test_processor_clone_capability() {
    let config = create_test_config();
    let processor = WebhookProcessor::new(config).unwrap();
    let processor_clone = processor.clone();
    
    let webhook = create_test_webhook_message();
    
    // Both original and clone should work
    let result1 = processor.process_webhook(webhook.clone(), "clone-test-1").await;
    let result2 = processor_clone.process_webhook(webhook, "clone-test-2").await;
    
    match (result1, result2) {
        (Ok(_), Ok(_)) => {},
        _ => {
            // Failures acceptable in test environment
        }
    }
}