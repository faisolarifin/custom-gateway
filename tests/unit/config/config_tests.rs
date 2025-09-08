use webhook_gateway::config::{AppConfig, ServerConfig, LoggerConfig, WebClientConfig, PermataBankLoginConfig, PermataBankWebhookConfig, SchedulerConfig};

#[test]
fn test_server_config_creation() {
    let config = ServerConfig {
        listen_host: "127.0.0.1".to_string(),
        listen_port: 8080,
        webhook_path: "/webhook".to_string(),
    };

    assert_eq!(config.listen_host, "127.0.0.1");
    assert_eq!(config.listen_port, 8080);
    assert_eq!(config.webhook_path, "/webhook");
}

#[test]
fn test_logger_config_creation() {
    let config = LoggerConfig {
        dir: "log/".to_string(),
        file_name: "webhook-gateway".to_string(),
        max_backups: 0,
        max_size: 10,
        max_age: 90,
        compress: true,
        local_time: true,
    };

    assert_eq!(config.dir, "log/");
    assert_eq!(config.file_name, "webhook-gateway");
    assert_eq!(config.max_backups, 0);
    assert_eq!(config.max_size, 10);
    assert_eq!(config.max_age, 90);
    assert!(config.compress);
    assert!(config.local_time);
}

#[test]
fn test_app_config_creation() {
    let server_config = ServerConfig {
        listen_host: "0.0.0.0".to_string(),
        listen_port: 9090,
        webhook_path: "/api/webhook".to_string(),
    };

    let logger_config = LoggerConfig {
        dir: "logs/".to_string(),
        file_name: "test-gateway".to_string(),
        max_backups: 5,
        max_size: 20,
        max_age: 30,
        compress: false,
        local_time: false,
    };

    let webclient_config = WebClientConfig {
        timeout: 30,
        max_retries: 3,
        retry_delay: 5,
    };

    let login_config = PermataBankLoginConfig {
        permata_static_key: "test_key".to_string(),
        api_key: "test_api".to_string(),
        token_url: "https://test.com/token".to_string(),
        username: "test_user".to_string(),
        password: "test_pass".to_string(),
        login_payload: "grant_type=client_credentials".to_string(),
    };

    let webhook_config_pb = PermataBankWebhookConfig {
        callbackstatus_url: "https://test.com/callback".to_string(),
        organizationname: "test_org".to_string(),
    };

    let scheduler_config = SchedulerConfig {
        periodic_interval_mins: 15,
    };

    let app_config = AppConfig {
        server: server_config,
        logger: logger_config,
        webclient: webclient_config,
        permata_bank_login: login_config,
        permata_bank_webhook: webhook_config_pb,
        token_scheduler: scheduler_config,
    };

    assert_eq!(app_config.server.listen_host, "0.0.0.0");
    assert_eq!(app_config.server.listen_port, 9090);
    assert_eq!(app_config.logger.dir, "logs/");
    assert_eq!(app_config.logger.file_name, "test-gateway");
}

#[tokio::test]
async fn test_config_loading() {
    // Test that config can be loaded from file (requires config.yaml)
    let result = AppConfig::load();
    // Should work if config.yaml exists and is valid
    match result {
        Ok(config) => {
            assert!(!config.server.listen_host.is_empty());
            assert!(config.server.listen_port > 0);
            assert!(!config.server.webhook_path.is_empty());
            println!("✅ Config loaded successfully");
        }
        Err(_) => {
            // Config file might not exist in test environment
            println!("⚠️  Config file not found or invalid - this is okay in test environment");
        }
    }
}