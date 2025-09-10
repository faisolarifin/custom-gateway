use webhook_gateway::models::*;
use serde_json::json;
use std::collections::HashMap;

#[test]
fn test_webhook_message_creation() {
    let mut headers = HashMap::new();
    headers.insert("content-type".to_string(), "application/json".to_string());
    headers.insert("authorization".to_string(), "Bearer token123".to_string());
    headers.insert("x-request-id".to_string(), "req-123456".to_string());
    
    let webhook_message = WebhookMessage {
        headers: headers.clone(),
        body: json!({"test": "data", "id": "webhook_123"}).to_string(),
    };
    
    assert_eq!(webhook_message.headers.len(), 3);
    assert!(webhook_message.headers.contains_key("content-type"));
    assert!(webhook_message.headers.contains_key("authorization"));
    assert!(webhook_message.headers.contains_key("x-request-id"));
    assert_eq!(webhook_message.headers.get("content-type").unwrap(), "application/json");
    assert!(webhook_message.body.contains("test"));
    assert!(webhook_message.body.contains("webhook_123"));
}

#[test]
fn test_webhook_message_serialization() {
    let mut headers = HashMap::new();
    headers.insert("content-type".to_string(), "application/json".to_string());
    
    let webhook_message = WebhookMessage {
        headers,
        body: json!({"message": "hello world"}).to_string(),
    };
    
    // Test serialization
    let serialized = serde_json::to_string(&webhook_message).unwrap();
    assert!(serialized.contains("headers"));
    assert!(serialized.contains("body"));
    assert!(serialized.contains("content-type"));
    assert!(serialized.contains("application/json"));
    
    // Test deserialization
    let deserialized: WebhookMessage = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.headers.len(), 1);
    assert!(deserialized.body.contains("hello world"));
}

#[test]
fn test_webhook_message_empty_headers() {
    let webhook_message = WebhookMessage {
        headers: HashMap::new(),
        body: "simple body".to_string(),
    };
    
    assert!(webhook_message.headers.is_empty());
    assert_eq!(webhook_message.body, "simple body");
}

#[test]
fn test_auth_request() {
    let auth_request = AuthRequest {
        username: "test_user".to_string(),
        password: "secure_password123".to_string(),
    };
    
    assert_eq!(auth_request.username, "test_user");
    assert_eq!(auth_request.password, "secure_password123");
    
    // Test serialization
    let serialized = serde_json::to_string(&auth_request).unwrap();
    assert!(serialized.contains("username"));
    assert!(serialized.contains("password"));
    assert!(serialized.contains("test_user"));
    
    // Test deserialization
    let deserialized: AuthRequest = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.username, "test_user");
    assert_eq!(deserialized.password, "secure_password123");
}

#[test]
fn test_auth_response() {
    let auth_response = AuthResponse {
        token: "jwt_token_abc123".to_string(),
        expires_in: Some(3600),
    };
    
    assert_eq!(auth_response.token, "jwt_token_abc123");
    assert_eq!(auth_response.expires_in, Some(3600));
    
    // Test with None expires_in
    let auth_response_no_expiry = AuthResponse {
        token: "permanent_token".to_string(),
        expires_in: None,
    };
    
    assert_eq!(auth_response_no_expiry.token, "permanent_token");
    assert_eq!(auth_response_no_expiry.expires_in, None);
}

#[test]
fn test_auth_response_serialization() {
    let auth_response = AuthResponse {
        token: "test_token_xyz".to_string(),
        expires_in: Some(7200),
    };
    
    let serialized = serde_json::to_string(&auth_response).unwrap();
    assert!(serialized.contains("token"));
    assert!(serialized.contains("expires_in"));
    assert!(serialized.contains("test_token_xyz"));
    assert!(serialized.contains("7200"));
    
    let deserialized: AuthResponse = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.token, "test_token_xyz");
    assert_eq!(deserialized.expires_in, Some(7200));
}

#[test]
fn test_token_response() {
    let token_response = TokenResponse {
        access_token: "access_token_123".to_string(),
        token_type: "Bearer".to_string(),
        expires_in: 3600,
        scope: "read write".to_string(),
    };
    
    assert_eq!(token_response.access_token, "access_token_123");
    assert_eq!(token_response.token_type, "Bearer");
    assert_eq!(token_response.expires_in, 3600);
    assert_eq!(token_response.scope, "read write");
}

#[test]
fn test_token_response_serialization() {
    let token_response = TokenResponse {
        access_token: "serialize_test_token".to_string(),
        token_type: "Bearer".to_string(),
        expires_in: 1800,
        scope: "api:read".to_string(),
    };
    
    let serialized = serde_json::to_string(&token_response).unwrap();
    assert!(serialized.contains("access_token"));
    assert!(serialized.contains("token_type"));
    assert!(serialized.contains("expires_in"));
    assert!(serialized.contains("scope"));
    
    let deserialized: TokenResponse = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.access_token, "serialize_test_token");
    assert_eq!(deserialized.token_type, "Bearer");
    assert_eq!(deserialized.expires_in, 1800);
    assert_eq!(deserialized.scope, "api:read");
}

#[test]
fn test_auth_context() {
    let expires_at = chrono::Utc::now();
    
    let auth_context = AuthContext {
        token: "context_token_456".to_string(),
        client_url: "https://api.example.com".to_string(),
        expires_at: Some(expires_at),
    };
    
    assert_eq!(auth_context.token, "context_token_456");
    assert_eq!(auth_context.client_url, "https://api.example.com");
    assert_eq!(auth_context.expires_at, Some(expires_at));
    
    // Test with None expires_at
    let auth_context_no_expiry = AuthContext {
        token: "permanent_context_token".to_string(),
        client_url: "https://permanent.example.com".to_string(),
        expires_at: None,
    };
    
    assert_eq!(auth_context_no_expiry.expires_at, None);
}

#[test]
fn test_webhook_payload() {
    let timestamp = chrono::Utc::now();
    let data = json!({"message_id": "msg_123", "content": "test message"});
    let changes = json!({"status": "delivered"});
    
    let webhook_payload = WebhookPayload {
        webhook_type: "message_status".to_string(),
        data: data.clone(),
        changes: Some(changes.clone()),
        timestamp,
    };
    
    assert_eq!(webhook_payload.webhook_type, "message_status");
    assert_eq!(webhook_payload.data, data);
    assert_eq!(webhook_payload.changes, Some(changes));
    assert_eq!(webhook_payload.timestamp, timestamp);
}

#[test]
fn test_webhook_payload_serialization() {
    let timestamp = chrono::DateTime::parse_from_rfc3339("2023-01-01T12:00:00Z")
        .unwrap()
        .with_timezone(&chrono::Utc);
    
    let webhook_payload = WebhookPayload {
        webhook_type: "delivery_receipt".to_string(),
        data: json!({"id": "dr_123"}),
        changes: None,
        timestamp,
    };
    
    let serialized = serde_json::to_string(&webhook_payload).unwrap();
    assert!(serialized.contains("webhook_type"));
    assert!(serialized.contains("data"));
    assert!(serialized.contains("timestamp"));
    assert!(serialized.contains("delivery_receipt"));
    
    let deserialized: WebhookPayload = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.webhook_type, "delivery_receipt");
    assert_eq!(deserialized.timestamp, timestamp);
}

#[test]
fn test_processing_result() {
    let timestamp = chrono::Utc::now();
    
    let processing_result = ProcessingResult {
        success: true,
        message: "Processing completed successfully".to_string(),
        request_id: "req-proc-123".to_string(),
        timestamp,
    };
    
    assert!(processing_result.success);
    assert_eq!(processing_result.message, "Processing completed successfully");
    assert_eq!(processing_result.request_id, "req-proc-123");
    assert_eq!(processing_result.timestamp, timestamp);
    
    // Test failure case
    let failure_result = ProcessingResult {
        success: false,
        message: "Processing failed with error".to_string(),
        request_id: "req-fail-456".to_string(),
        timestamp,
    };
    
    assert!(!failure_result.success);
    assert!(failure_result.message.contains("failed"));
}

#[test]
fn test_processing_result_serialization() {
    let timestamp = chrono::DateTime::parse_from_rfc3339("2023-01-01T15:30:00Z")
        .unwrap()
        .with_timezone(&chrono::Utc);
    
    let processing_result = ProcessingResult {
        success: false,
        message: "Authentication error".to_string(),
        request_id: "req-auth-error".to_string(),
        timestamp,
    };
    
    let serialized = serde_json::to_string(&processing_result).unwrap();
    assert!(serialized.contains("success"));
    assert!(serialized.contains("message"));
    assert!(serialized.contains("request_id"));
    assert!(serialized.contains("timestamp"));
    
    let deserialized: ProcessingResult = serde_json::from_str(&serialized).unwrap();
    assert!(!deserialized.success);
    assert_eq!(deserialized.message, "Authentication error");
    assert_eq!(deserialized.request_id, "req-auth-error");
}

#[test]
fn test_permata_webhook_response() {
    let response = PermataWebhookResponse {
        status_code: "00".to_string(),
        status_desc: "Success".to_string(),
    };
    
    assert_eq!(response.status_code, "00");
    assert_eq!(response.status_desc, "Success");
    
    // Test error response
    let error_response = PermataWebhookResponse {
        status_code: "99".to_string(),
        status_desc: "Internal Server Error".to_string(),
    };
    
    assert_eq!(error_response.status_code, "99");
    assert_eq!(error_response.status_desc, "Internal Server Error");
}

#[test]
fn test_permata_webhook_response_serialization() {
    let response = PermataWebhookResponse {
        status_code: "06".to_string(),
        status_desc: "Request timeout".to_string(),
    };
    
    let serialized = serde_json::to_string(&response).unwrap();
    // Should use renamed fields
    assert!(serialized.contains("StatusCode"));
    assert!(serialized.contains("StatusDesc"));
    assert!(serialized.contains("06"));
    assert!(serialized.contains("Request timeout"));
    
    // Test deserialization with renamed fields
    let json_str = r#"{"StatusCode": "00", "StatusDesc": "OK"}"#;
    let deserialized: PermataWebhookResponse = serde_json::from_str(json_str).unwrap();
    assert_eq!(deserialized.status_code, "00");
    assert_eq!(deserialized.status_desc, "OK");
}

// Integration tests with real-world JSON payloads
#[test]
fn test_webhook_message_with_real_payload() {
    let mut headers = HashMap::new();
    headers.insert("content-type".to_string(), "application/json".to_string());
    headers.insert("x-hub-signature".to_string(), "sha256=abc123".to_string());
    
    let dr_payload = json!({
        "status": {
            "id": "wamid.HBgNNjI4MTMzNzE4NTQwFQIAERgSREE4NDFFMEY1RkY0NDlEMzEA",
            "status": "delivered",
            "timestamp": "1234567890",
            "recipient_id": "628133718540"
        }
    });
    
    let webhook_message = WebhookMessage {
        headers,
        body: dr_payload.to_string(),
    };
    
    assert!(webhook_message.body.contains("delivered"));
    assert!(webhook_message.body.contains("wamid"));
    assert!(webhook_message.headers.contains_key("x-hub-signature"));
}

#[test]
fn test_model_cloning() {
    let original_auth = AuthRequest {
        username: "original_user".to_string(),
        password: "original_pass".to_string(),
    };
    
    let cloned_auth = original_auth.clone();
    assert_eq!(original_auth.username, cloned_auth.username);
    assert_eq!(original_auth.password, cloned_auth.password);
    
    // Test that they are independent
    drop(original_auth);
    assert_eq!(cloned_auth.username, "original_user");
}

#[test]
fn test_model_debug_formatting() {
    let token_response = TokenResponse {
        access_token: "debug_token".to_string(),
        token_type: "Bearer".to_string(),
        expires_in: 900,
        scope: "debug".to_string(),
    };
    
    let debug_str = format!("{:?}", token_response);
    assert!(debug_str.contains("TokenResponse"));
    assert!(debug_str.contains("debug_token"));
    assert!(debug_str.contains("Bearer"));
    assert!(debug_str.contains("900"));
}

// Edge cases and error conditions
#[test]
fn test_empty_string_fields() {
    let empty_auth = AuthRequest {
        username: "".to_string(),
        password: "".to_string(),
    };
    
    assert!(empty_auth.username.is_empty());
    assert!(empty_auth.password.is_empty());
    
    // Should still serialize/deserialize correctly
    let serialized = serde_json::to_string(&empty_auth).unwrap();
    let deserialized: AuthRequest = serde_json::from_str(&serialized).unwrap();
    assert!(deserialized.username.is_empty());
    assert!(deserialized.password.is_empty());
}