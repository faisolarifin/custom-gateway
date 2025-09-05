use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("HTTP request error: {0}")]
    HttpRequest(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),

    #[error("Authentication failed: {message}")]
    AuthenticationFailed { message: String },

    #[error("Message processing error: {message}")]
    MessageProcessing { message: String },

    #[error("Payload conversion error: {message}")]
    PayloadConversion { message: String },

    #[error("Webhook type error: {message}")]
    WebhookType { message: String },

    #[error("Configuration error: {message}")]
    Configuration { message: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Logging initialization error: {0}")]
    LoggingInit(#[from] tracing_appender::rolling::InitError),

    #[error("Generic error: {0}")]
    Generic(#[from] anyhow::Error),

    #[error("HMAC error: {0}")]
    Hmac(#[from] hmac::digest::InvalidLength),
}

impl AppError {
    pub fn authentication_failed(message: impl Into<String>) -> Self {
        Self::AuthenticationFailed {
            message: message.into(),
        }
    }

    pub fn message_processing(message: impl Into<String>) -> Self {
        Self::MessageProcessing {
            message: message.into(),
        }
    }

    pub fn payload_conversion(message: impl Into<String>) -> Self {
        Self::PayloadConversion {
            message: message.into(),
        }
    }

    pub fn webhook_type(message: impl Into<String>) -> Self {
        Self::WebhookType {
            message: message.into(),
        }
    }

    pub fn configuration(message: impl Into<String>) -> Self {
        Self::Configuration {
            message: message.into(),
        }
    }
}

pub type Result<T> = std::result::Result<T, AppError>;