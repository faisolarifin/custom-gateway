use webhook_gateway::utils::request_id::extract_request_id;
use serde_json::json;
use uuid::Uuid;

#[test]
fn test_extract_request_id_with_xid() {
    let payload = json!({
        "xid": "test_xid_123",
        "data": "some data",
        "timestamp": 1234567890
    }).to_string();
    
    let request_id = extract_request_id(&payload);
    
    assert!(request_id.starts_with("req-"));
    assert!(request_id.contains("test_xid_123"));
    assert_eq!(request_id, "req-test_xid_123");
}

#[test]
fn test_extract_request_id_with_id_when_no_xid() {
    let payload = json!({
        "id": "test_id_456",
        "data": "some data",
        "timestamp": 1234567890
    }).to_string();
    
    let request_id = extract_request_id(&payload);
    
    assert!(request_id.starts_with("req-"));
    assert!(request_id.contains("test_id_456"));
    assert_eq!(request_id, "req-test_id_456");
}

#[test]
fn test_extract_request_id_prefers_xid_over_id() {
    let payload = json!({
        "xid": "priority_xid",
        "id": "secondary_id",
        "data": "some data"
    }).to_string();
    
    let request_id = extract_request_id(&payload);
    
    assert!(request_id.starts_with("req-"));
    assert!(request_id.contains("priority_xid"));
    assert!(!request_id.contains("secondary_id"));
    assert_eq!(request_id, "req-priority_xid");
}

#[test]
fn test_extract_request_id_with_empty_xid_falls_back_to_id() {
    let payload = json!({
        "xid": "",
        "id": "fallback_id",
        "data": "some data"
    }).to_string();
    
    let request_id = extract_request_id(&payload);
    
    assert!(request_id.starts_with("req-"));
    assert!(request_id.contains("fallback_id"));
    assert_eq!(request_id, "req-fallback_id");
}

#[test]
fn test_extract_request_id_with_empty_id_generates_uuid() {
    let payload = json!({
        "id": "",
        "data": "some data"
    }).to_string();
    
    let request_id = extract_request_id(&payload);
    
    assert!(request_id.starts_with("req-"));
    assert!(request_id.len() > 10); // Should contain a UUID
    
    // Extract the UUID part and verify it's valid
    let uuid_part = &request_id[4..]; // Remove "req-" prefix
    assert!(Uuid::parse_str(uuid_part).is_ok());
}

#[test]
fn test_extract_request_id_no_ids_generates_uuid() {
    let payload = json!({
        "data": "some data",
        "timestamp": 1234567890,
        "status": "success"
    }).to_string();
    
    let request_id = extract_request_id(&payload);
    
    assert!(request_id.starts_with("req-"));
    assert!(request_id.len() > 10); // Should contain a UUID
    
    // Extract the UUID part and verify it's valid
    let uuid_part = &request_id[4..]; // Remove "req-" prefix
    assert!(Uuid::parse_str(uuid_part).is_ok());
}

#[test]
fn test_extract_request_id_invalid_json_generates_uuid() {
    let invalid_json = r#"{"incomplete": json"#;
    
    let request_id = extract_request_id(invalid_json);
    
    assert!(request_id.starts_with("req-"));
    assert!(request_id.len() > 10); // Should contain a UUID
    
    // Extract the UUID part and verify it's valid
    let uuid_part = &request_id[4..]; // Remove "req-" prefix
    assert!(Uuid::parse_str(uuid_part).is_ok());
}

#[test]
fn test_extract_request_id_empty_string_generates_uuid() {
    let empty_payload = "";
    
    let request_id = extract_request_id(empty_payload);
    
    assert!(request_id.starts_with("req-"));
    assert!(request_id.len() > 10); // Should contain a UUID
    
    // Extract the UUID part and verify it's valid
    let uuid_part = &request_id[4..]; // Remove "req-" prefix
    assert!(Uuid::parse_str(uuid_part).is_ok());
}

#[test]
fn test_extract_request_id_non_string_ids() {
    // Test with numeric id
    let payload = json!({
        "id": 123456,
        "data": "some data"
    }).to_string();
    
    let request_id = extract_request_id(&payload);
    
    // Should generate UUID since id is not a string
    assert!(request_id.starts_with("req-"));
    assert!(request_id.len() > 10); // Should contain a UUID
    
    let uuid_part = &request_id[4..]; // Remove "req-" prefix
    assert!(Uuid::parse_str(uuid_part).is_ok());
}

#[test]
fn test_extract_request_id_null_ids() {
    let payload = json!({
        "xid": null,
        "id": null,
        "data": "some data"
    }).to_string();
    
    let request_id = extract_request_id(&payload);
    
    // Should generate UUID since ids are null
    assert!(request_id.starts_with("req-"));
    assert!(request_id.len() > 10); // Should contain a UUID
    
    let uuid_part = &request_id[4..]; // Remove "req-" prefix
    assert!(Uuid::parse_str(uuid_part).is_ok());
}

#[test]
fn test_extract_request_id_special_characters() {
    let payload = json!({
        "xid": "test-id_with.special@chars",
        "data": "some data"
    }).to_string();
    
    let request_id = extract_request_id(&payload);
    
    assert!(request_id.starts_with("req-"));
    assert!(request_id.contains("test-id_with.special@chars"));
    assert_eq!(request_id, "req-test-id_with.special@chars");
}

#[test]
fn test_extract_request_id_unicode_characters() {
    let payload = json!({
        "xid": "тест_id_ñoël",
        "data": "some data"
    }).to_string();
    
    let request_id = extract_request_id(&payload);
    
    assert!(request_id.starts_with("req-"));
    assert!(request_id.contains("тест_id_ñoël"));
    assert_eq!(request_id, "req-тест_id_ñoël");
}

#[test]
fn test_extract_request_id_very_long_id() {
    let very_long_id = "a".repeat(1000);
    let payload = json!({
        "xid": very_long_id.clone(),
        "data": "some data"
    }).to_string();
    
    let request_id = extract_request_id(&payload);
    
    assert!(request_id.starts_with("req-"));
    assert!(request_id.contains(&very_long_id));
    assert_eq!(request_id, format!("req-{}", very_long_id));
}

#[test]
fn test_extract_request_id_whitespace_ids() {
    // Test with whitespace-only xid - the current implementation uses it as-is (doesn't trim)
    let payload = json!({
        "xid": "   ",
        "id": "valid_id",
        "data": "some data"
    }).to_string();
    
    let request_id = extract_request_id(&payload);
    
    assert!(request_id.starts_with("req-"));
    // The implementation uses the whitespace xid since it's not empty
    assert_eq!(request_id, "req-   ");
    
    // Test with empty xid, should fallback to id
    let payload_empty_xid = json!({
        "xid": "",
        "id": "valid_id",
        "data": "some data"
    }).to_string();
    
    let request_id_fallback = extract_request_id(&payload_empty_xid);
    assert!(request_id_fallback.starts_with("req-"));
    assert!(request_id_fallback.contains("valid_id"));
    assert_eq!(request_id_fallback, "req-valid_id");
}

#[test]
fn test_extract_request_id_consistency() {
    let payload = json!({
        "xid": "consistent_id_123",
        "data": "some data"
    }).to_string();
    
    // Multiple calls should return the same result
    let request_id1 = extract_request_id(&payload);
    let request_id2 = extract_request_id(&payload);
    
    assert_eq!(request_id1, request_id2);
    assert_eq!(request_id1, "req-consistent_id_123");
}

#[test]
fn test_extract_request_id_uuid_generation_uniqueness() {
    let payload = json!({
        "data": "no id fields"
    }).to_string();
    
    // Multiple calls should generate different UUIDs
    let request_id1 = extract_request_id(&payload);
    let request_id2 = extract_request_id(&payload);
    
    assert!(request_id1.starts_with("req-"));
    assert!(request_id2.starts_with("req-"));
    assert_ne!(request_id1, request_id2);
    
    // Both should be valid UUIDs
    let uuid1 = &request_id1[4..];
    let uuid2 = &request_id2[4..];
    assert!(Uuid::parse_str(uuid1).is_ok());
    assert!(Uuid::parse_str(uuid2).is_ok());
}

// Integration test with real webhook payloads
#[test]
fn test_extract_request_id_webhook_examples() {
    // DR (Delivery Receipt) payload example
    let dr_payload = json!({
        "status": {
            "id": "wamid.HBgNNjI4MTMzNzE4NTQwFQIAERgSREE4NDFFMEY1RkY0NDlEMzEA",
            "status": "delivered",
            "timestamp": "1234567890",
            "recipient_id": "628133718540"
        }
    }).to_string();
    
    let request_id = extract_request_id(&dr_payload);
    // Should generate UUID since there's no top-level xid or id
    assert!(request_id.starts_with("req-"));
    assert!(request_id.len() > 10);
    
    // Inbound Flow payload example
    let inbound_payload = json!({
        "xid": "webhook_123456",
        "messages": [{
            "id": "wamid.HBgNNjI4MTMzNzE4NTQwFQIAERgSREE4NDFFMEY1RkY0NDlEMzEA",
            "from": "628133718540",
            "timestamp": "1234567890",
            "text": {
                "body": "Hello, this is a test message"
            },
            "type": "text"
        }]
    }).to_string();
    
    let request_id = extract_request_id(&inbound_payload);
    assert_eq!(request_id, "req-webhook_123456");
}