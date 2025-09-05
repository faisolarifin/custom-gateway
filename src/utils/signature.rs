use hmac::{Hmac, Mac};
use sha2::Sha256;
use base64::Engine;

use crate::utils::error::Result;

type HmacSha256 = Hmac<Sha256>;

pub fn generate_signature(static_key: &str, key: &str, timestamp: &str, data: &str) -> Result<String> {
    let message = format!("{}:{}:{}", key, timestamp, data);

    let mut mac = HmacSha256::new_from_slice(static_key.as_bytes())?;
    mac.update(message.as_bytes());
    
    let result = mac.finalize();
    let signature = base64::engine::general_purpose::STANDARD.encode(result.into_bytes());
    
    Ok(signature)
}