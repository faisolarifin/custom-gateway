use webhook_gateway::config::{AppConfig, PermataBankLoginConfig, WebClientConfig, ServerConfig, PermataBankWebhookConfig, SchedulerConfig, TelegramAlertConfig, LoggerConfig};
use webhook_gateway::services::LoginHandler;
use mockito::Server;
use serde_json::json;
use tokio::time::{timeout, Duration};

fn create_test_config(mock_server_url: &str) -> AppConfig {
    AppConfig {
        server: ServerConfig {
            listen_host: "127.0.0.1".to_string(),
            listen_port: 8080,
            webhook_path: "/webhook".to_string(),
        },
        webclient: WebClientConfig {
            timeout: 30,
            max_retries: 3,
            retry_delay: 1, // Use shorter delay for tests
        },
        permata_bank_login: PermataBankLoginConfig {
            permata_static_key: "test_key".to_string(),
            api_key: "test_api_key".to_string(),
            token_url: format!("{}/token", mock_server_url),
            username: "test_user".to_string(),
            password: "test_pass".to_string(),
            login_payload: "grant_type=client_credentials".to_string(),
        },
        permata_bank_webhook: PermataBankWebhookConfig {
            callbackstatus_url: format!("{}/callback", mock_server_url),
            organizationname: "test_org".to_string(),
        },
        token_scheduler: SchedulerConfig {
            periodic_interval_mins: 15,
        },
        telegram_alert: TelegramAlertConfig {
            api_url: format!("{}/bot123:test/sendMessage", mock_server_url),
            chat_id: "-123456789".to_string(),
            message_thread_id: "123".to_string(),
            alert_message_prefix: "[TEST]".to_string(),
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
}

#[tokio::test]
async fn test_login_handler_creation() {
    let server = Server::new_async().await;
    let config = create_test_config(&server.url());
    let handler = LoginHandler::new(config);
    assert!(handler.is_ok());
}

#[tokio::test]
async fn test_login_handler_successful_login() {
    let mut server = Server::new_async().await;
    
    // Mock successful token response - expect multiple calls due to scheduler
    let token_mock = server.mock("POST", "/token")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(json!({
            "access_token": "test_access_token_123",
            "token_type": "Bearer",
            "expires_in": 3600,
            "scope": "api"
        }).to_string())
        .expect_at_least(1)
        .create_async().await;
    
    let config = create_test_config(&server.url());
    let handler = LoginHandler::new(config).unwrap();
    
    let result = timeout(Duration::from_secs(10), handler.get_token()).await;
    assert!(result.is_ok());
    
    let token = result.unwrap().unwrap();
    assert_eq!(token, "test_access_token_123");
    
    token_mock.assert_async().await;
    
    // Test shutdown
    handler.shutdown().await;
}

#[tokio::test]
async fn test_login_handler_token_caching() {
    let mut server = Server::new_async().await;
    
    // Mock should handle multiple calls due to background scheduler
    let token_mock = server.mock("POST", "/token")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(json!({
            "access_token": "cached_token_456",
            "token_type": "Bearer",
            "expires_in": 3600,
            "scope": "api"
        }).to_string())
        .expect_at_least(1)
        .create_async().await;
    
    let config = create_test_config(&server.url());
    let handler = LoginHandler::new(config).unwrap();
    
    // First call should hit the mock
    let token1 = handler.get_token().await.unwrap();
    assert_eq!(token1, "cached_token_456");
    
    // Second call should use cached token
    let token2 = handler.get_token().await.unwrap();
    assert_eq!(token2, "cached_token_456");
    
    token_mock.assert_async().await;
    handler.shutdown().await;
}

#[tokio::test]
async fn test_login_handler_cache_clear() {
    let server = Server::new_async().await;
    let config = create_test_config(&server.url());
    let handler = LoginHandler::new(config).unwrap();
    
    // This should not panic
    handler.clear_cache();
    
    // Test with context
    handler.clear_cache_with_context(Some("unique_id"), Some("req_id"));
    
    handler.shutdown().await;
}

#[tokio::test]
async fn test_login_handler_auth_failure() {
    let mut server = Server::new_async().await;
    
    // Mock authentication failure - expect multiple calls due to retries and scheduler
    let token_mock = server.mock("POST", "/token")
        .with_status(401)
        .with_header("content-type", "application/json")
        .with_body(json!({
            "error": "unauthorized",
            "error_description": "Invalid credentials"
        }).to_string())
        .expect_at_least(1)
        .create_async().await;
    
    let config = create_test_config(&server.url());
    let handler = LoginHandler::new(config).unwrap();
    
    let result = handler.get_token().await;
    assert!(result.is_err());
    
    let error = result.unwrap_err();
    // Test expects ReqError which is what the login handler actually returns
    assert!(matches!(error, webhook_gateway::utils::error::AppError::ReqError { .. }));
    
    token_mock.assert_async().await;
    handler.shutdown().await;
}

#[tokio::test]
async fn test_login_handler_retry_mechanism() {
    let mut server = Server::new_async().await;
    
    // Mock for retry mechanism - expect multiple calls
    let retry_mock = server.mock("POST", "/token")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(json!({
            "access_token": "retry_success_token",
            "token_type": "Bearer",
            "expires_in": 3600,
            "scope": "api"
        }).to_string())
        .expect_at_least(1)
        .create_async().await;
    
    let config = create_test_config(&server.url());
    let handler = LoginHandler::new(config).unwrap();
    
    let result = handler.get_token().await;
    assert!(result.is_ok());
    
    let token = result.unwrap();
    assert_eq!(token, "retry_success_token");
    
    retry_mock.assert_async().await;
    
    handler.shutdown().await;
}

#[tokio::test]
async fn test_login_handler_scheduler_control() {
    let server = Server::new_async().await;
    let config = create_test_config(&server.url());
    let handler = LoginHandler::new(config).unwrap();
    
    // Test scheduler status methods
    assert!(handler.is_scheduler_active());
    
    let info = handler.get_scheduler_info();
    assert!(info.is_some());
    assert!(info.unwrap().contains("active"));
    
    // Stop scheduler
    handler.stop_scheduler();
    assert!(!handler.is_scheduler_active());
    assert!(handler.get_scheduler_info().is_none());
    
    handler.shutdown().await;
}

#[tokio::test]
async fn test_login_handler_with_context() {
    let mut server = Server::new_async().await;
    
    let token_mock = server.mock("POST", "/token")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(json!({
            "access_token": "context_token_789",
            "token_type": "Bearer",
            "expires_in": 3600,
            "scope": "api"
        }).to_string())
        .expect_at_least(1)
        .create_async().await;
    
    let config = create_test_config(&server.url());
    let handler = LoginHandler::new(config).unwrap();
    
    let result = handler.get_token_with_context(Some("unique_123"), Some("req_456")).await;
    assert!(result.is_ok());
    
    let token = result.unwrap();
    assert_eq!(token, "context_token_789");
    
    token_mock.assert_async().await;
    handler.shutdown().await;
}