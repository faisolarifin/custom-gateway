use webhook_gateway::config::{AppConfig, PermataBankLoginConfig, WebClientConfig, ServerConfig, PermataBankWebhookConfig, LoggerConfig};
use webhook_gateway::services::LoginHandler;

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
            retry_delay: 1, // Use shorter delay for tests
        },
        permata_bank_login: PermataBankLoginConfig {
            permata_static_key: "test_key".to_string(),
            api_key: "test_api_key".to_string(),
            token_url: "https://httpbin.org/post".to_string(), // Use httpbin for testing
            username: "test_user".to_string(),
            password: "test_pass".to_string(),
            login_payload: "grant_type=client_credentials".to_string(),
            oauth_timestamp: "2024-04-25T13:52:01.000+07:00".to_string(),
        },
        permata_bank_webhook: PermataBankWebhookConfig {
            callbackstatus_url: "https://example.com".to_string(),
            organizationname: "test_org".to_string(),
            permata_timestamp: "2024-04-25T13:52:01.000+07:00".to_string(),
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
    let config = create_test_config();
    let handler = LoginHandler::new(config);
    assert!(handler.is_ok());
}

#[test]
fn test_login_handler_cache_clear() {
    let config = create_test_config();
    let handler = LoginHandler::new(config).unwrap();
    
    // This should not panic
    handler.clear_cache();
}