// Simple test to verify configuration loading
use webhook_gateway::config::AppConfig;

pub fn test_config_loading() -> anyhow::Result<()> {
    println!("Testing configuration loading...");
    
    let config = AppConfig::load()?;
    
    println!("✓ Configuration loaded successfully");
    println!("  - Server host: {}", config.server.listen_host);
    println!("  - Server port: {}", config.server.listen_port);
    println!("  - Webhook path: {}", config.server.webhook_path);
    println!("  - Permata Callback URL: {}", config.permata_bank_webhook.callbackstatus_url);
    println!("  - Timeout: {}s", config.webclient.timeout);
    println!("  - Logger dir: {}", config.logger.dir);
    println!("  - Logger file: {}", config.logger.file_name);
    
    Ok(())
}

pub fn test_message_processor() -> anyhow::Result<()> {
    println!("\nTesting message processor...");
    
    use webhook_gateway::services::WebhookProcessor;
    use webhook_gateway::models::WebhookMessage;
    use std::collections::HashMap;
    
    let config = webhook_gateway::config::AppConfig::load()?;
    let _processor = WebhookProcessor::new(config)?;
    println!("✓ Message processor created successfully");
    
    // Create a test webhook message
    let mut headers = HashMap::new();
    headers.insert("content-type".to_string(), "application/json".to_string());
    headers.insert("user-agent".to_string(), "test-client/1.0".to_string());
    
    let webhook = WebhookMessage {
        headers,
        body: r#"{"test": "data", "timestamp": "2024-01-01T00:00:00Z"}"#.to_string(),
    };
    
    println!("✓ Test webhook message created");
    println!("  - Body size: {} bytes", webhook.body.len());
    println!("  - Headers count: {}", webhook.headers.len());
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config() {
        test_config_loading().expect("Config test failed");
    }

    #[tokio::test] 
    async fn test_processor() {
        test_message_processor().expect("Processor test failed");
    }
}