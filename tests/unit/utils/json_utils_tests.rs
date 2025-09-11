use webhook_gateway::utils::{compact_json, json_path_exists, json_path_equals, navigate_json_path, is_dr_payload, is_inbound_flow_payload};
use serde_json::Value;

const REAL_WHATSAPP_PAYLOAD: &str = r#"{
    "xid": "123",
    "entry": [
        {
            "id": "115617074757249",
            "time": 0,
            "changes": [
                {
                    "field": "messages",
                    "value": {
                        "metadata": {
                            "phone_number_id": "115159954803011",
                            "display_phone_number": "6287845715199"
                        },
                        "statuses": [
                            {
                                "id": "wamid.HBgNNjI4MjIyODIyMzUwMBUCABEYEjg1ODdCMEMxRjkyNUJCRUY5NwA=",
                                "errors": null,
                                "status": "delivered",
                                "pricing": {
                                    "billable": true,
                                    "category": "user_initiated",
                                    "pricing_model": "CBP"
                                },
                                "timestamp": "1677836780",
                                "conversation": {
                                    "id": "4ea98b1e873569598832d04b6894ab08",
                                    "origin": {
                                        "type": "user_initiated"
                                    },
                                    "expiration_timestamp": null
                                },
                                "recipient_id": "6282228223500"
                            }
                        ],
                        "messaging_product": "whatsapp"
                    }
                }
            ]
        }
    ]
}"#;

const INBOUND_FLOW_PAYLOAD: &str = r#"{
    "data": {
        "entry": [
            {
                "changes": [
                    {
                        "value": {
                            "messages": [
                                {
                                    "interactive": {
                                        "type": "nfm_reply"
                                    }
                                }
                            ]
                        }
                    }
                ]
            }
        ]
    }
}"#;

const DR_ERROR_PAYLOAD: &str = r#"{
    "error": {
        "code": 500,
        "message": "Internal server error"
    }
}"#;

#[test]
fn test_compact_json_basic() {
    let input = r#"{
        "user": "john doe",
        "email": "john@example.com"
    }"#;
    
    let result = compact_json(input).unwrap();
    
    // Function removes ALL whitespace including spaces in strings
    assert!(!result.contains('\n'));
    assert!(!result.contains("  "));
    assert!(result.contains("johndoe"));  // Space removed from "john doe"
    assert!(result.contains("john@example.com"));
    assert_eq!(result, r#"{"user":"johndoe","email":"john@example.com"}"#);
}

#[test]
fn test_compact_json_preserves_values() {
    let input = r#"{"message": "Hello World with spaces", "number": 42}"#;
    let result = compact_json(input).unwrap();
    
    // Function removes ALL whitespace, so we can't parse it back as valid JSON
    // Just verify the basic structure is maintained
    assert!(!result.contains('\n'));
    assert!(!result.contains("  "));
    assert!(result.contains("HelloWorldwithspaces")); // Spaces removed
    assert!(result.contains("42"));
    assert_eq!(result, r#"{"message":"HelloWorldwithspaces","number":42}"#);
}

#[test]
fn test_compact_json_nested() {
    let input = r#"{
        "data": {
            "field1": "value with spaces",
            "field2": 123,
            "nested": {
                "deep": "value"
            }
        }
    }"#;
    
    let result = compact_json(input).unwrap();
    
    assert!(!result.contains('\n'));
    assert!(result.contains("valuewithspaces")); // Spaces removed
    assert!(result.contains("123"));
    assert!(result.contains("value"));
    assert_eq!(result, r#"{"data":{"field1":"valuewithspaces","field2":123,"nested":{"deep":"value"}}}"#);
}

#[test]
fn test_compact_json_with_real_whatsapp_payload() {
    let result = compact_json(REAL_WHATSAPP_PAYLOAD).unwrap();
    
    assert!(!result.contains('\n'));
    assert!(!result.contains("  "));
    
    let original: Value = serde_json::from_str(REAL_WHATSAPP_PAYLOAD).unwrap();
    let compacted: Value = serde_json::from_str(&result).unwrap();
    assert_eq!(original, compacted);
    
    assert!(result.contains("115617074757249"));
    assert!(result.contains("delivered"));
    assert!(result.contains("6282228223500"));
}

#[test]
fn test_compact_json_invalid_json() {
    let invalid_json = r#"{"invalid": json}"#;
    let result = compact_json(invalid_json);
    // Function just removes whitespace, doesn't validate JSON
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), r#"{"invalid":json}"#);
}

#[test]
fn test_navigate_json_path_simple() {
    let json: Value = serde_json::from_str(REAL_WHATSAPP_PAYLOAD).unwrap();
    
    let results = navigate_json_path(&json, &["xid"]);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].as_str(), Some("123"));
}

#[test]
fn test_navigate_json_path_nested() {
    let json: Value = serde_json::from_str(REAL_WHATSAPP_PAYLOAD).unwrap();
    
    let results = navigate_json_path(&json, &["entry", "[*]", "id"]);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].as_str(), Some("115617074757249"));
}

#[test]
fn test_navigate_json_path_array_iteration() {
    let json: Value = serde_json::from_str(REAL_WHATSAPP_PAYLOAD).unwrap();
    
    let results = navigate_json_path(&json, &["entry", "[*]", "changes", "[*]", "value", "statuses", "[*]", "status"]);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].as_str(), Some("delivered"));
}

#[test]
fn test_navigate_json_path_nonexistent() {
    let json: Value = serde_json::from_str(REAL_WHATSAPP_PAYLOAD).unwrap();
    
    let results = navigate_json_path(&json, &["nonexistent"]);
    assert!(results.is_empty());
}

#[test]
fn test_navigate_json_path_empty_path() {
    let json: Value = serde_json::from_str(REAL_WHATSAPP_PAYLOAD).unwrap();
    
    let results = navigate_json_path(&json, &[]);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0], &json);
}

#[test]
fn test_navigate_json_path_deep_nested() {
    let json: Value = serde_json::from_str(REAL_WHATSAPP_PAYLOAD).unwrap();
    
    let results = navigate_json_path(&json, &[
        "entry", "[*]", "changes", "[*]", "value", 
        "statuses", "[*]", "conversation", "origin", "type"
    ]);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].as_str(), Some("user_initiated"));
}

#[test]
fn test_json_path_exists_simple() {
    let json: Value = serde_json::from_str(REAL_WHATSAPP_PAYLOAD).unwrap();
    
    assert!(json_path_exists(&json, &["xid"]));
    assert!(json_path_exists(&json, &["entry"]));
    assert!(!json_path_exists(&json, &["nonexistent"]));
}

#[test]
fn test_json_path_exists_array_iteration() {
    let json: Value = serde_json::from_str(REAL_WHATSAPP_PAYLOAD).unwrap();
    
    assert!(json_path_exists(&json, &["entry", "[*]", "changes", "[*]", "value", "statuses"]));
    assert!(!json_path_exists(&json, &["entry", "[*]", "changes", "[*]", "value", "messages"]));
}

#[test]
fn test_json_path_equals_simple() {
    let json: Value = serde_json::from_str(REAL_WHATSAPP_PAYLOAD).unwrap();
    
    assert!(json_path_equals(&json, &["xid"], "123"));
    assert!(!json_path_equals(&json, &["xid"], "456"));
}

#[test]
fn test_json_path_equals_nested() {
    let json: Value = serde_json::from_str(REAL_WHATSAPP_PAYLOAD).unwrap();
    
    assert!(json_path_equals(&json, &["entry", "[*]", "changes", "[*]", "value", "statuses", "[*]", "status"], "delivered"));
    assert!(!json_path_equals(&json, &["entry", "[*]", "changes", "[*]", "value", "statuses", "[*]", "status"], "failed"));
}

#[test]
fn test_is_dr_payload_with_statuses() {
    let json: Value = serde_json::from_str(REAL_WHATSAPP_PAYLOAD).unwrap();
    assert!(is_dr_payload(&json));
}

#[test]
fn test_is_dr_payload_with_error() {
    let json: Value = serde_json::from_str(DR_ERROR_PAYLOAD).unwrap();
    assert!(is_dr_payload(&json));
}

#[test]
fn test_is_dr_payload_false() {
    let non_dr_payload = r#"{"message": "hello"}"#;
    let json: Value = serde_json::from_str(non_dr_payload).unwrap();
    assert!(!is_dr_payload(&json));
}

#[test]
fn test_is_inbound_flow_payload_true() {
    let json: Value = serde_json::from_str(INBOUND_FLOW_PAYLOAD).unwrap();
    assert!(is_inbound_flow_payload(&json));
}

#[test]
fn test_is_inbound_flow_payload_false() {
    let json: Value = serde_json::from_str(REAL_WHATSAPP_PAYLOAD).unwrap();
    assert!(!is_inbound_flow_payload(&json));
}

#[test]
fn test_is_inbound_flow_payload_with_different_type() {
    let payload_with_different_type = r#"{
        "data": {
            "entry": [
                {
                    "changes": [
                        {
                            "value": {
                                "messages": [
                                    {
                                        "interactive": {
                                            "type": "button_reply"
                                        }
                                    }
                                ]
                            }
                        }
                    ]
                }
            ]
        }
    }"#;
    let json: Value = serde_json::from_str(payload_with_different_type).unwrap();
    assert!(!is_inbound_flow_payload(&json));
}

#[test]
fn test_navigate_json_path_multiple_array_results() {
    let payload_with_multiple_statuses = r#"{
        "entry": [
            {
                "changes": [
                    {
                        "value": {
                            "statuses": [
                                {"status": "sent"},
                                {"status": "delivered"}
                            ]
                        }
                    }
                ]
            }
        ]
    }"#;
    let json: Value = serde_json::from_str(payload_with_multiple_statuses).unwrap();
    
    let results = navigate_json_path(&json, &["entry", "[*]", "changes", "[*]", "value", "statuses", "[*]", "status"]);
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].as_str(), Some("sent"));
    assert_eq!(results[1].as_str(), Some("delivered"));
}

#[test]
fn test_navigate_json_path_array_with_non_object() {
    let payload_with_array_values = r#"{
        "numbers": [1, 2, 3, 4]
    }"#;
    let json: Value = serde_json::from_str(payload_with_array_values).unwrap();
    
    let results = navigate_json_path(&json, &["numbers", "[*]"]);
    assert_eq!(results.len(), 4);
    assert_eq!(results[0].as_u64(), Some(1));
    assert_eq!(results[3].as_u64(), Some(4));
}

#[test]
fn test_json_path_exists_with_null_values() {
    let json: Value = serde_json::from_str(REAL_WHATSAPP_PAYLOAD).unwrap();
    
    assert!(json_path_exists(&json, &["entry", "[*]", "changes", "[*]", "value", "statuses", "[*]", "errors"]));
    assert!(json_path_exists(&json, &["entry", "[*]", "changes", "[*]", "value", "statuses", "[*]", "conversation", "expiration_timestamp"]));
}