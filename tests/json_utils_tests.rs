use webhook_gateway::utils::compact_json;

#[test]
fn test_compact_json_basic() {
    let input = r#"{
        "user": "john doe",
        "email": "john@example.com"
    }"#;
    
    let result = compact_json(input).unwrap();
    
    // Should not contain newlines or unnecessary spaces
    assert!(!result.contains('\n'));
    assert!(!result.contains("  "));
    
    // Should still contain the actual values
    assert!(result.contains("john doe"));
    assert!(result.contains("john@example.com"));
}

#[test]
fn test_compact_json_preserves_values() {
    let input = r#"{"message": "Hello World with spaces", "number": 42}"#;
    let result = compact_json(input).unwrap();
    
    // Parse both to ensure they're equivalent
    let original_value: serde_json::Value = serde_json::from_str(input).unwrap();
    let compacted_value: serde_json::Value = serde_json::from_str(&result).unwrap();
    
    assert_eq!(original_value, compacted_value);
    assert!(result.contains("Hello World with spaces"));
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
    assert!(result.contains("value with spaces"));
    assert!(result.contains("123"));
    assert!(result.contains("value"));
}