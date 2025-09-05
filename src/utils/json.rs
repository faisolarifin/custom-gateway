use crate::utils::error::Result;

/// Compacts a JSON string by removing unnecessary whitespace and newlines
/// while preserving the original values
pub fn compact_json(json_str: &str) -> Result<String> {
    let value: serde_json::Value = serde_json::from_str(json_str)?;
    let compacted = serde_json::to_string(&value)?;
    Ok(compacted)
}