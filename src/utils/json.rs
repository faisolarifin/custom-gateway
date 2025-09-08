use crate::utils::error::Result;

/// Compacts a JSON string by removing unnecessary whitespace and newlines
/// while preserving the original values
pub fn compact_json(json_str: &str) -> Result<String> {
    let value: serde_json::Value = serde_json::from_str(json_str)?;
    let compacted = serde_json::to_string(&value)?;
    Ok(compacted)
}

/// Checks if a JSON path exists in the given JSON value
/// Supports array iteration with "[*]" syntax
pub fn json_path_exists(json: &serde_json::Value, path: &[&str]) -> bool {
    !navigate_json_path(json, path).is_empty()
}

/// Checks if a JSON path equals the expected value
/// Supports array iteration with "[*]" syntax
pub fn json_path_equals(json: &serde_json::Value, path: &[&str], expected_value: &str) -> bool {
    navigate_json_path(json, path)
        .iter()
        .any(|value| value.as_str() == Some(expected_value))
}

/// Navigates through a JSON structure using a path array
/// Returns all matching values at the end of the path
/// Supports "[*]" for array iteration
pub fn navigate_json_path<'a>(current: &'a serde_json::Value, path: &[&str]) -> Vec<&'a serde_json::Value> {
    if path.is_empty() {
        return vec![current];
    }

    let segment = path[0];
    let remaining_path = &path[1..];

    match segment {
        "[*]" => {
            // Handle array iteration
            let mut results = Vec::new();
            if let Some(array) = current.as_array() {
                for item in array {
                    results.extend(navigate_json_path(item, remaining_path));
                }
            }
            results
        }
        field_name => {
            // Handle object field access
            if let Some(field_value) = current.get(field_name) {
                navigate_json_path(field_value, remaining_path)
            } else {
                vec![]
            }
        }
    }
}

/// Checks if the given JSON payload is a DR (Delivery Receipt) payload
/// DR payloads have either:
/// 1. An "error" field (for error messages)
/// 2. entry.changes.value.statuses field (for status messages)
pub fn is_dr_payload(json: &serde_json::Value) -> bool {
    // Check for DR error message (has "error" field)
    if json.get("error").is_some() {
        return true;
    }
    
    // Check for DR status message using JSONPath-like approach
    json_path_exists(json, &["entry", "[*]", "changes", "[*]", "value", "statuses"])
}

/// Checks if the given JSON payload is an Inbound Flow payload
/// Inbound Flow payloads have:
/// data.entry.changes.value.messages.interactive.type = "nfm_reply"
pub fn is_inbound_flow_payload(json: &serde_json::Value) -> bool {
    // Check for Inbound Flow using JSONPath-like approach
    json_path_equals(
        json,
        &["data", "entry", "[*]", "changes", "[*]", "value", "messages", "[*]", "interactive", "type"],
        "nfm_reply"
    )
}