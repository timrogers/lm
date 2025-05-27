use thiserror::Error;

/// Errors that can occur when using the La Marzocco client
#[derive(Error, Debug)]
pub enum LaMarzoccoError {
    /// Authentication failed due to invalid credentials
    #[error("Authentication failed: {0}")]
    AuthenticationError(String),

    /// Error making HTTP request
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    /// Error with config file
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Error when serializing or deserializing
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_yaml::Error),

    /// Error with IO operations
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// API returned non-success response
    #[error("API error: {status_code} - {message}")]
    ApiError {
        status_code: u16,
        message: String,
    },

    /// Other errors
    #[error("Other error: {0}")]
    OtherError(String),
}

pub type Result<T> = std::result::Result<T, LaMarzoccoError>;