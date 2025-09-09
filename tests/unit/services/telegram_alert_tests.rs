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
            token_url: "test".to_string(),
            username: "test".to_string(),
            password: "test".to_string(),
            login_payload: "test".to_string(),
        },
        permata_bank_webhook: PermataBankWebhookConfig {
            callbackstatus_url: "test".to_string(),
            organizationname: "test".to_string(),
        },
        token_scheduler: SchedulerConfig {
            periodic_interval_mins: 3,
        },
        telegram_alert: TelegramAlertConfig {
            api_url: "https://api.telegram.org/bot5801394322:AAEaWtt-jGFb81sT7KqrUOw1Pg9m9dtkWas/sendMessage".to_string(),
            chat_id: "-1001904746324".to_string(),
            message_thread_id: "140801".to_string(),
            alert_message_prefix: "[TEST ALERT]".to_string(),
        },
        logger: LoggerConfig {
            dir: "log/".to_string(),
            file_name: "test".to_string(),
            max_backups: 0,
            max_size: 10,
            max_age: 90,
            compress: true,
            local_time: true,
        },
    }
}

#[test]
fn test_telegram_service_creation() {
    let config = create_test_config();
    let service = TelegramAlertService::new(config);
    assert!(service.is_ok());
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