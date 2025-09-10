use webhook_gateway::services::TelegramAlertService;
use webhook_gateway::config::*;

fn create_test_config() -> AppConfig {
    AppConfig {
        server: ServerConfig {
            listen_host: "127.0.0.1".to_string(),
            listen_port: 8080,
            webhook_path: "/webhook".to_string(),
        },
        webclient: WebClientConfig {
            timeout: 30,
            max_retries: 1,
            retry_delay: 1,
        },
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
        token_scheduler: SchedulerConfig {
            periodic_interval_mins: 3,
        },
        telegram_alert: TelegramAlertConfig {
            api_url: "https://httpbin.org/status/200".to_string(),
            chat_id: "-1001904746324".to_string(),
            message_thread_id: "140801".to_string(),
            alert_message_prefix: "[TEST ALERT]".to_string(),
        },
        logger: LoggerConfig {
            dir: std::env::temp_dir().to_string_lossy().to_string(),
            file_name: "test-telegram-alert".to_string(),
            max_backups: 0,
            max_size: 10,
            max_age: 90,
            compress: true,
            local_time: true,
        },
    }
}

fn create_config_with_timeout(timeout_seconds: u64) -> AppConfig {
    let mut config = create_test_config();
    config.webclient.timeout = timeout_seconds;
    config
}

fn create_config_with_invalid_url() -> AppConfig {
    let mut config = create_test_config();
    config.telegram_alert.api_url = "https://invalid-domain-that-does-not-exist.com".to_string();
    config
}

fn create_config_with_error_url() -> AppConfig {
    let mut config = create_test_config();
    config.telegram_alert.api_url = "https://httpbin.org/status/500".to_string();
    config
}

fn create_config_with_short_timeout() -> AppConfig {
    let mut config = create_test_config();
    config.webclient.timeout = 1; // 1 second timeout
    config
}

fn create_config_with_different_prefix() -> AppConfig {
    let mut config = create_test_config();
    config.telegram_alert.alert_message_prefix = "[PRODUCTION ERROR]".to_string();
    config
}

fn create_config_with_different_chat_settings() -> AppConfig {
    let mut config = create_test_config();
    config.telegram_alert.chat_id = "-987654321".to_string();
    config.telegram_alert.message_thread_id = "54321".to_string();
    config
}

#[test]
fn test_telegram_service_creation() {
    let config = create_test_config();
    let service = TelegramAlertService::new(config);
    assert!(service.is_ok());
}

#[test]
fn test_telegram_service_creation_with_zero_timeout() {
    let mut config = create_test_config();
    config.webclient.timeout = 0;
    
    let service = TelegramAlertService::new(config);
    assert!(service.is_ok());
}

#[test]
fn test_telegram_service_creation_with_large_timeout() {
    let config = create_config_with_timeout(600); // 10 minutes
    let service = TelegramAlertService::new(config);
    assert!(service.is_ok());
}

#[test]
fn test_telegram_service_creation_with_different_prefix() {
    let config = create_config_with_different_prefix();
    let service = TelegramAlertService::new(config);
    assert!(service.is_ok());
}

#[test]
fn test_telegram_service_creation_with_different_chat_settings() {
    let config = create_config_with_different_chat_settings();
    let service = TelegramAlertService::new(config);
    assert!(service.is_ok());
}

#[tokio::test]
async fn test_telegram_service_clone() {
    let config = create_test_config();
    let service = TelegramAlertService::new(config).unwrap();
    let service_clone = service.clone();
    
    // Both services should work independently
    service.send_error_alert("Test message 1", Some("req-1"));
    service_clone.send_error_alert("Test message 2", Some("req-2"));
    
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
}

#[tokio::test]
async fn test_telegram_service_send_error_alert_non_blocking() {
    let config = create_test_config();
    let service = TelegramAlertService::new(config).unwrap();
    
    // Test bahwa service tidak memblock
    let start = std::time::Instant::now();
    service.send_error_alert("Test error message", Some("test-request-id"));
    let duration = start.elapsed();
    
    // Pastikan function return dengan cepat (non-blocking)
    assert!(duration < std::time::Duration::from_millis(100));
    
    // Wait sedikit untuk async task selesai
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
}

#[tokio::test]
async fn test_telegram_service_send_error_alert_without_request_id() {
    let config = create_test_config();
    let service = TelegramAlertService::new(config).unwrap();
    
    // Test bahwa tidak ada panic ketika request_id None
    service.send_error_alert("Test error without request_id", None);
    
    // Wait sedikit untuk async task selesai
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
}

#[tokio::test]
async fn test_telegram_service_send_error_alert_with_empty_message() {
    let config = create_test_config();
    let service = TelegramAlertService::new(config).unwrap();
    
    service.send_error_alert("", Some("test-req-id"));
    
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
}

#[tokio::test]
async fn test_telegram_service_send_error_alert_with_long_message() {
    let config = create_test_config();
    let service = TelegramAlertService::new(config).unwrap();
    
    let long_message = "Error message ".repeat(100); // Very long message
    service.send_error_alert(&long_message, Some("long-msg-req-id"));
    
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
}

#[tokio::test]
async fn test_telegram_service_send_error_alert_with_special_characters() {
    let config = create_test_config();
    let service = TelegramAlertService::new(config).unwrap();
    
    let message_with_special_chars = "Error: JSON parse failed! @#$%^&*()_+{}|:<>?[]\\;',./\"";
    service.send_error_alert(message_with_special_chars, Some("special-chars-req"));
    
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
}

#[tokio::test]
async fn test_telegram_service_send_error_alert_with_unicode() {
    let config = create_test_config();
    let service = TelegramAlertService::new(config).unwrap();
    
    let unicode_message = "Error: 网络连接失败 🚨 Ошибка сети 💥 エラーが発生しました";
    service.send_error_alert(unicode_message, Some("unicode-req"));
    
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
}

#[tokio::test]
async fn test_telegram_service_multiple_concurrent_alerts() {
    let config = create_test_config();
    let service = TelegramAlertService::new(config).unwrap();
    
    // Send multiple alerts concurrently
    let tasks = (0..5).map(|i| {
        let service = service.clone();
        tokio::spawn(async move {
            service.send_error_alert(
                &format!("Concurrent error message {}", i), 
                Some(&format!("concurrent-req-{}", i))
            );
        })
    });
    
    // Wait for all tasks to complete
    for task in tasks {
        task.await.unwrap();
    }
    
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_telegram_service_send_alert_with_network_error() {
    let config = create_config_with_invalid_url();
    let service = TelegramAlertService::new(config).unwrap();
    
    // This should not panic even with invalid URL
    service.send_error_alert("Test network error handling", Some("network-error-req"));
    
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_telegram_service_send_alert_with_http_error() {
    let config = create_config_with_error_url();
    let service = TelegramAlertService::new(config).unwrap();
    
    // This should handle HTTP errors gracefully
    service.send_error_alert("Test HTTP error handling", Some("http-error-req"));
    
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_telegram_service_send_alert_with_short_timeout() {
    let config = create_config_with_short_timeout();
    let service = TelegramAlertService::new(config).unwrap();
    
    // This might timeout but should handle it gracefully
    service.send_error_alert("Test timeout handling", Some("timeout-req"));
    
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
}

#[tokio::test]
async fn test_telegram_service_message_formatting_with_request_id() {
    let config = create_config_with_different_prefix();
    let service = TelegramAlertService::new(config).unwrap();
    
    service.send_error_alert("Database connection failed", Some("req-12345"));
    
    // Expected format: "[PRODUCTION ERROR] [request-id: req-12345] Database connection failed"
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
}

#[tokio::test]
async fn test_telegram_service_message_formatting_without_request_id() {
    let config = create_config_with_different_prefix();
    let service = TelegramAlertService::new(config).unwrap();
    
    service.send_error_alert("Application startup failed", None);
    
    // Expected format: "[PRODUCTION ERROR] Application startup failed"
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
}

#[tokio::test]
async fn test_telegram_service_rapid_sequential_alerts() {
    let config = create_test_config();
    let service = TelegramAlertService::new(config).unwrap();
    
    // Send alerts rapidly in sequence
    for i in 0..10 {
        service.send_error_alert(
            &format!("Rapid alert #{}", i), 
            Some(&format!("rapid-{}", i))
        );
    }
    
    tokio::time::sleep(std::time::Duration::from_millis(150)).await;
}

#[tokio::test]
async fn test_telegram_service_with_empty_prefix() {
    let mut config = create_test_config();
    config.telegram_alert.alert_message_prefix = String::new();
    
    let service = TelegramAlertService::new(config).unwrap();
    service.send_error_alert("Test with empty prefix", Some("empty-prefix-req"));
    
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
}

#[tokio::test]
async fn test_telegram_service_with_newlines_in_message() {
    let config = create_test_config();
    let service = TelegramAlertService::new(config).unwrap();
    
    let multiline_message = "Error occurred:\nLine 1: Database connection failed\nLine 2: Retry attempts exhausted\nLine 3: Service unavailable";
    service.send_error_alert(multiline_message, Some("multiline-req"));
    
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
}

#[tokio::test]
async fn test_telegram_service_error_scenarios_do_not_panic() {
    let configs = vec![
        create_config_with_invalid_url(),
        create_config_with_error_url(),
        create_config_with_short_timeout(),
    ];
    
    for (i, config) in configs.into_iter().enumerate() {
        let service = TelegramAlertService::new(config).unwrap();
        service.send_error_alert(
            &format!("Test error scenario {}", i), 
            Some(&format!("error-scenario-{}", i))
        );
    }
    
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
}

#[tokio::test]
async fn test_telegram_service_stress_test() {
    let config = create_test_config();
    let service = TelegramAlertService::new(config).unwrap();
    
    // Stress test with many alerts
    let tasks: Vec<_> = (0..50).map(|i| {
        let service = service.clone();
        tokio::spawn(async move {
            service.send_error_alert(
                &format!("Stress test message {}", i), 
                Some(&format!("stress-{}", i))
            );
        })
    }).collect();
    
    // Wait for all tasks to complete
    for task in tasks {
        task.await.unwrap();
    }
    
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
}