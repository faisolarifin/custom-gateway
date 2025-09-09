use uuid::Uuid;

pub fn extract_request_id(payload: &str) -> String {
    match serde_json::from_str::<serde_json::Value>(payload) {
        Ok(json) => {
            // Try to get xid first, then id
            if let Some(xid) = json.get("xid").and_then(|v| v.as_str()) {
                if !xid.is_empty() {
                    return format!("req-{}", xid);
                }
            }
            
            if let Some(id) = json.get("id").and_then(|v| v.as_str()) {
                if !id.is_empty() {
                    return format!("req-{}", id);
                }
            }
            
            // Generate UUID if no xid or id found
            format!("req-{}", Uuid::new_v4())
        }
        Err(_) => {
            // Generate UUID if payload is not valid JSON
            format!("req-{}", Uuid::new_v4())
        }
    }
}