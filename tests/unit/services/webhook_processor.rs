use std::collections::HashMap;
use webhook_gateway::{
    config::*,
    models::WebhookMessage,
    services::{WebhookProcessor, WebhookProcessorTrait},
};

fn create_test_webhook_message() -> WebhookMessage {
    let mut headers = HashMap::new();
    headers.insert("content-type".to_string(), "application/json".to_string());
    headers.insert("user-agent".to_string(), "test-client/1.0".to_string());
    
    WebhookMessage {
        headers,
        body: r#"{"user": "john", "email": "john@example.com"}"#.to_string(),
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

#[tokio::test]
async fn test_webhook_processor_creation() {
    let config = create_test_config();
    let processor = WebhookProcessor::new(config);
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

#[test]
fn test_config_validation() {
    let config = create_test_config();
    assert_eq!(config.server.listen_host, "127.0.0.1");
    assert_eq!(config.server.listen_port, 8080);
    assert_eq!(config.permata_bank_webhook.organizationname, "test_org");
    assert!(config.permata_bank_webhook.callbackstatus_url.starts_with("https://"));
}

#[tokio::test]
async fn test_authentication_error_handling() {
    
    // Create a config that will fail authentication (invalid credentials)
    let mut config = create_test_config();
    config.permata_bank_login.token_url = "https://httpbin.org/status/401".to_string(); // Will return 401
    
    let processor = WebhookProcessor::new(config).unwrap();
    let webhook = create_test_webhook_message();
    
    // This should handle the auth failure gracefully and return Ok(())
    let result = processor.process_webhook(webhook, "test-auth-failure").await;
    
    // The result should be Ok because auth failures are handled gracefully
    assert!(result.is_ok(), "Authentication failures should be handled gracefully");
}