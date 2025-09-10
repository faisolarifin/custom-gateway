use webhook_gateway::utils::error::{AppError, Result};
use std::io;

#[test]
fn test_app_error_authentication_failed() {
    let error = AppError::authentication_failed("Invalid credentials");
    
    match &error {
        AppError::AuthenticationFailed { message } => {
            assert_eq!(message, "Invalid credentials");
        }
        _ => panic!("Expected AuthenticationFailed variant"),
    }
    
    let error_string = format!("{}", error);
    assert!(error_string.contains("Authentication failed"));
    assert!(error_string.contains("Invalid credentials"));
}

#[test]
fn test_app_error_message_processing() {
    let error = AppError::message_processing("Failed to parse message");
    
    match &error {
        AppError::MessageProcessing { message } => {
            assert_eq!(message, "Failed to parse message");
        }
        _ => panic!("Expected MessageProcessing variant"),
    }
    
    let error_string = format!("{}", error);
    assert!(error_string.contains("Message processing error"));
    assert!(error_string.contains("Failed to parse message"));
}

#[test]
fn test_app_error_payload_conversion() {
    let error = AppError::payload_conversion("Invalid payload format");
    
    match &error {
        AppError::PayloadConversion { message } => {
            assert_eq!(message, "Invalid payload format");
        }
        _ => panic!("Expected PayloadConversion variant"),
    }
    
    let error_string = format!("{}", error);
    assert!(error_string.contains("Payload conversion error"));
    assert!(error_string.contains("Invalid payload format"));
}

#[test]
fn test_app_error_webhook_type() {
    let error = AppError::webhook_type("Unsupported webhook type");
    
    match &error {
        AppError::WebhookType { message } => {
            assert_eq!(message, "Unsupported webhook type");
        }
        _ => panic!("Expected WebhookType variant"),
    }
    
    let error_string = format!("{}", error);
    assert!(error_string.contains("Webhook type error"));
    assert!(error_string.contains("Unsupported webhook type"));
}

#[test]
fn test_app_error_configuration() {
    let error = AppError::configuration("Missing required config");
    
    match &error {
        AppError::Configuration { message } => {
            assert_eq!(message, "Missing required config");
        }
        _ => panic!("Expected Configuration variant"),
    }
    
    let error_string = format!("{}", error);
    assert!(error_string.contains("Configuration error"));
    assert!(error_string.contains("Missing required config"));
}

#[test]
fn test_app_error_generic_error() {
    let error = AppError::error("Generic error message");
    
    match &error {
        AppError::ReqError { message } => {
            assert_eq!(message, "Generic error message");
        }
        _ => panic!("Expected ReqError variant"),
    }
    
    let error_string = format!("{}", error);
    assert_eq!(error_string, "Generic error message");
}

#[tokio::test]
async fn test_app_error_from_reqwest_error() {
    // Test with a real reqwest error from invalid URL
    let client = reqwest::Client::new();
    let result = client.get("invalid-url").send().await;
    assert!(result.is_err());
    
    let reqwest_error = result.unwrap_err();
    let app_error: AppError = reqwest_error.into();
    
    match app_error {
        AppError::HttpRequest(_) => {
            // Expected variant
        }
        _ => panic!("Expected HttpRequest variant"),
    }
    
    let error_string = format!("{}", app_error);
    assert!(error_string.contains("HTTP request error"));
}

#[test]
fn test_app_error_from_serde_json_error() {
    let json_str = r#"{"invalid": json"#;
    let json_error = serde_json::from_str::<serde_json::Value>(json_str).unwrap_err();
    let app_error: AppError = json_error.into();
    
    match app_error {
        AppError::Serialization(_) => {
            // Expected variant
        }
        _ => panic!("Expected Serialization variant"),
    }
    
    let error_string = format!("{}", app_error);
    assert!(error_string.contains("Serialization error"));
}

#[test]
fn test_app_error_from_io_error() {
    let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
    let app_error: AppError = io_error.into();
    
    match app_error {
        AppError::Io(_) => {
            // Expected variant
        }
        _ => panic!("Expected Io variant"),
    }
    
    let error_string = format!("{}", app_error);
    assert!(error_string.contains("IO error"));
    assert!(error_string.contains("File not found"));
}

#[test]
fn test_app_error_from_hmac_error() {
    let hmac_error = hmac::digest::InvalidLength;
    let app_error: AppError = hmac_error.into();
    
    match app_error {
        AppError::Hmac(_) => {
            // Expected variant
        }
        _ => panic!("Expected Hmac variant"),
    }
    
    let error_string = format!("{}", app_error);
    assert!(error_string.contains("HMAC error"));
}

#[test]
fn test_app_error_from_anyhow_error() {
    let anyhow_error = anyhow::anyhow!("Custom anyhow error");
    let app_error: AppError = anyhow_error.into();
    
    match app_error {
        AppError::Generic(_) => {
            // Expected variant
        }
        _ => panic!("Expected Generic variant"),
    }
    
    let error_string = format!("{}", app_error);
    assert!(error_string.contains("Generic error"));
    assert!(error_string.contains("Custom anyhow error"));
}

#[test]
fn test_result_type_ok() {
    let success_result: Result<String> = Ok("Success".to_string());
    assert!(success_result.is_ok());
    assert_eq!(success_result.unwrap(), "Success");
}

#[test]
fn test_result_type_err() {
    let error_result: Result<String> = Err(AppError::error("Test error"));
    assert!(error_result.is_err());
    
    let error = error_result.unwrap_err();
    assert_eq!(format!("{}", error), "Test error");
}

#[test]
fn test_error_chaining_and_conversion() {
    // Test error propagation through ? operator
    fn inner_function() -> Result<String> {
        let json_str = r#"{"invalid": json"#;
        let _parsed: serde_json::Value = serde_json::from_str(json_str)?;
        Ok("Success".to_string())
    }
    
    let result = inner_function();
    assert!(result.is_err());
    
    let error = result.unwrap_err();
    assert!(matches!(error, AppError::Serialization(_)));
}

#[test]
fn test_error_debug_trait() {
    let error = AppError::authentication_failed("Debug test");
    let debug_str = format!("{:?}", error);
    
    assert!(debug_str.contains("AuthenticationFailed"));
    assert!(debug_str.contains("Debug test"));
}

#[test]
fn test_multiple_error_constructor_forms() {
    // Test with &str
    let error1 = AppError::authentication_failed("str message");
    assert!(format!("{}", error1).contains("str message"));
    
    // Test with String
    let error2 = AppError::authentication_failed("String message".to_string());
    assert!(format!("{}", error2).contains("String message"));
    
    // Test with formatted string
    let error3 = AppError::authentication_failed(format!("Formatted {}", "message"));
    assert!(format!("{}", error3).contains("Formatted message"));
}

#[test]
fn test_error_equality_by_message() {
    let error1 = AppError::error("Same message");
    let error2 = AppError::error("Same message");
    let error3 = AppError::error("Different message");
    
    // Since AppError doesn't implement PartialEq, we compare string representations
    assert_eq!(format!("{}", error1), format!("{}", error2));
    assert_ne!(format!("{}", error1), format!("{}", error3));
}

// Integration test with real-world scenario
#[tokio::test]
async fn test_error_in_async_context() {
    async fn async_operation() -> Result<String> {
        // Simulate an authentication failure
        Err(AppError::authentication_failed("Token expired"))
    }
    
    let result = async_operation().await;
    assert!(result.is_err());
    
    let error = result.unwrap_err();
    match error {
        AppError::AuthenticationFailed { message } => {
            assert_eq!(message, "Token expired");
        }
        _ => panic!("Expected AuthenticationFailed"),
    }
}