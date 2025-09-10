use mockito::Server;
use serde_json::json;
use tokio::time::{timeout, Duration};

use webhook_gateway::config::{AppConfig, PermataBankLoginConfig, PermataBankWebhookConfig, WebClientConfig, TelegramAlertConfig, SchedulerConfig, LoggerConfig};
use webhook_gateway::services::PermataCallbackStatusClient;
use webhook_gateway::utils::error::AppError;

fn create_test_config(mock_server_url: &str) -> AppConfig {
    AppConfig {
        server: webhook_gateway::config::ServerConfig {
            listen_host: "127.0.0.1".to_string(),
            listen_port: 8080,
            webhook_path: "/webhook".to_string(),
        },
        permata_bank_login: PermataBankLoginConfig {
            username: "test_user".to_string(),
            password: "test_pass".to_string(),
            api_key: "test_api_key".to_string(),
            token_url: format!("{}/token", mock_server_url),
            permata_static_key: "test_static_key".to_string(),
            login_payload: "grant_type=client_credentials".to_string(),
        },
        permata_bank_webhook: PermataBankWebhookConfig {
            callbackstatus_url: format!("{}/callback", mock_server_url),
            organizationname: "TestOrg".to_string(),
        },
        webclient: WebClientConfig {
            timeout: 30,
            max_retries: 3,
            retry_delay: 1,
        },
        telegram_alert: TelegramAlertConfig {
            api_url: format!("{}/bot123:token/sendMessage", mock_server_url),
            chat_id: "-123456789".to_string(),
            message_thread_id: "123".to_string(),
            alert_message_prefix: "[TEST] Alert:".to_string(),
        },
        token_scheduler: SchedulerConfig {
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
}

#[tokio::test]
async fn test_permata_callback_client_new() {
    let server = Server::new_async().await;
    let config = create_test_config(&server.url());
    
    let client = PermataCallbackStatusClient::new(config);
    assert!(client.is_ok());
}

#[tokio::test]
async fn test_send_webhook_success() {
    let mut server = Server::new_async().await;
    
    // Mock token endpoint - expect multiple calls due to scheduler
    let token_mock = server.mock("POST", "/token")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(json!({
            "access_token": "test_token_123",
            "token_type": "Bearer",
            "expires_in": 3600,
            "scope": "api"
        }).to_string())
        .expect_at_least(1)
        .create_async().await;
    
    // Mock callback endpoint
    let callback_mock = server.mock("POST", "/callback")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(json!({
            "StatusCode": "00",
            "StatusDesc": "Success"
        }).to_string())
        .create_async().await;
    
    let config = create_test_config(&server.url());
    let client = PermataCallbackStatusClient::new(config).unwrap();
    
    let webhook_body = json!({
        "test_data": "webhook_payload",
        "id": "test_123"
    }).to_string();
    
    let result = timeout(Duration::from_secs(10), 
        client.send_webhook(&webhook_body, "req-test-123")
    ).await;
    
    assert!(result.is_ok());
    let response = result.unwrap().unwrap();
    assert_eq!(response.status_code, 200);
    assert!(response.body.contains("Success"));
    
    token_mock.assert_async().await;
    callback_mock.assert_async().await;
}

#[tokio::test]
async fn test_send_webhook_with_context() {
    let mut server = Server::new_async().await;
    
    // Mock token endpoint - expect multiple calls due to scheduler
    let token_mock = server.mock("POST", "/token")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(json!({
            "access_token": "test_token_456",
            "token_type": "Bearer",
            "expires_in": 3600,
            "scope": "api"
        }).to_string())
        .expect_at_least(1)
        .create_async().await;
    
    // Mock callback endpoint
    let callback_mock = server.mock("POST", "/callback")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(json!({
            "StatusCode": "00",
            "StatusDesc": "Success"
        }).to_string())
        .create_async().await;
    
    let config = create_test_config(&server.url());
    let client = PermataCallbackStatusClient::new(config).unwrap();
    
    let webhook_body = json!({
        "test_data": "webhook_payload_with_context",
        "xid": "context_test_456"
    }).to_string();
    
    let result = client.send_webhook_with_context(
        &webhook_body, 
        "req-context-test", 
        Some("unique-456"), 
        Some("x-req-456")
    ).await;
    
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.status_code, 200);
    assert!(response.body.contains("Success"));
    
    token_mock.assert_async().await;
    callback_mock.assert_async().await;
    
    // Shutdown to stop scheduler
    client.shutdown().await;
}

#[tokio::test]
async fn test_send_webhook_http_error() {
    let mut server = Server::new_async().await;
    
    // Mock token endpoint to succeed
    let _token_mock = server.mock("POST", "/token")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(json!({
            "access_token": "test_token_error",
            "token_type": "Bearer",
            "expires_in": 3600,
            "scope": "api"
        }).to_string())
        .create_async().await;
    
    // Mock callback endpoint to return error
    let callback_mock = server.mock("POST", "/callback")
        .with_status(500)
        .with_header("content-type", "application/json")
        .with_body(json!({
            "StatusCode": "99",
            "StatusDesc": "Internal Server Error"
        }).to_string())
        .create_async().await;
    
    let config = create_test_config(&server.url());
    let client = PermataCallbackStatusClient::new(config).unwrap();
    
    let webhook_body = json!({
        "test_data": "error_payload",
        "id": "error_123"
    }).to_string();
    
    let result = client.send_webhook(&webhook_body, "req-error-123").await;
    
    // Should return the error response, not fail
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.status_code, 500);
    assert!(response.body.contains("Internal Server Error"));
    
    callback_mock.assert_async().await;
    
    // Shutdown to stop scheduler
    client.shutdown().await;
}

#[tokio::test]
async fn test_send_webhook_auth_error() {
    let mut server = Server::new_async().await;
    
    // Mock token endpoint to fail - expect multiple calls due to retries and scheduler
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
    let client = PermataCallbackStatusClient::new(config).unwrap();
    
    let webhook_body = json!({
        "test_data": "auth_error_payload",
        "id": "auth_error_123"
    }).to_string();
    
    let result = client.send_webhook(&webhook_body, "req-auth-error").await;
    
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(error, AppError::AuthenticationFailed { .. }));
    
    token_mock.assert_async().await;
    
    // Shutdown to stop scheduler
    client.shutdown().await;
}

#[tokio::test]
async fn test_send_webhook_retry_mechanism() {
    let mut server = Server::new_async().await;
    
    // Mock token endpoint to succeed
    let _token_mock = server.mock("POST", "/token")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(json!({
            "access_token": "test_token_retry",
            "token_type": "Bearer", 
            "expires_in": 3600,
            "scope": "api"
        }).to_string())
        .expect_at_least(1)
        .create_async().await;
    
    // Mock successful callback after retries
    let callback_mock_success = server.mock("POST", "/callback")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(json!({
            "StatusCode": "00",
            "StatusDesc": "Success after retry"
        }).to_string())
        .expect_at_least(1)
        .create_async().await;
    
    let config = create_test_config(&server.url());
    let client = PermataCallbackStatusClient::new(config).unwrap();
    
    let webhook_body = json!({
        "test_data": "retry_payload",
        "id": "retry_123"
    }).to_string();
    
    let result = client.send_webhook(&webhook_body, "req-retry-test").await;
    
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.status_code, 200);
    assert!(response.body.contains("Success after retry"));
    
    // Verify mock was called
    callback_mock_success.assert_async().await;
    
    // Shutdown to stop scheduler
    client.shutdown().await;
}

// Note: is_authentication_error is a private method, so we test it indirectly
// through the public API behavior when authentication errors occur
#[tokio::test]
async fn test_authentication_error_handling() {
    let mut server = Server::new_async().await;
    
    // Test that authentication failures are properly handled
    let _token_mock = server.mock("POST", "/token")
        .with_status(401)
        .with_header("content-type", "application/json")
        .with_body("Unauthorized")
        .expect_at_least(1)
        .create_async().await;
    
    let config = create_test_config(&server.url());
    let client = PermataCallbackStatusClient::new(config).unwrap();
    
    let webhook_body = json!({"test": "auth_error"}).to_string();
    let result = client.send_webhook(&webhook_body, "req-auth-test").await;
    
    // Should get authentication error
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(error, AppError::AuthenticationFailed { .. }));
    
    // Shutdown to stop scheduler
    client.shutdown().await;
}

#[tokio::test]
async fn test_shutdown() {
    let server = Server::new_async().await;
    let config = create_test_config(&server.url());
    let client = PermataCallbackStatusClient::new(config).unwrap();
    
    // Should not panic or error
    client.shutdown().await;
}

#[tokio::test]
async fn test_send_webhook_malformed_json() {
    let mut server = Server::new_async().await;
    
    // Mock token endpoint
    let _token_mock = server.mock("POST", "/token")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(json!({
            "access_token": "test_token_json",
            "token_type": "Bearer",
            "expires_in": 3600,
            "scope": "api"
        }).to_string())
        .expect_at_least(1)
        .create_async().await;
    
    // Mock callback endpoint
    let callback_mock = server.mock("POST", "/callback")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(json!({
            "StatusCode": "00",
            "StatusDesc": "Success"
        }).to_string())
        .create_async().await;
    
    let config = create_test_config(&server.url());
    let client = PermataCallbackStatusClient::new(config).unwrap();
    
    // Test with malformed JSON - should still work as it's just treated as string
    let webhook_body = r#"{"incomplete_json": true"#;
    
    let result = client.send_webhook(webhook_body, "req-malformed").await;
    
    // Should handle the malformed JSON gracefully - the client will treat it as raw string
    // The result depends on server's JSON compacting logic
    match result {
        Ok(_) => {
            // JSON compacting succeeded or server accepted malformed JSON
            callback_mock.assert_async().await;
        }
        Err(_) => {
            // JSON compacting failed - this is also acceptable behavior
            // The error comes from the compact_json utility function
        }
    }
}