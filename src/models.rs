use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Authentication response from the La Marzocco API
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthResponse {
    /// Access token for API requests
    #[serde(rename = "accessToken")]
    pub access_token: String,
    /// Refresh token for getting new access tokens
    #[serde(rename = "refreshToken")]
    pub refresh_token: String,
    /// Username (email) of the authenticated user
    pub username: String,
}

/// Stored credentials in the config file
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Credentials {
    /// Username (email) for authentication
    pub username: String,
    /// Access token for API requests
    pub access_token: String,
    /// Refresh token for getting new access tokens
    pub refresh_token: String,
    /// When the token expires
    pub expires_at: DateTime<Utc>,
}

/// Machine data from the La Marzocco API
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Machine {
    /// Serial number of the machine
    pub serial_number: String,
    /// Model name
    pub model_name: String,
    /// Whether the machine is turned on
    pub turned_on: bool,
    /// Whether the machine is ready
    pub is_ready: bool,
}

/// Dashboard configuration from the La Marzocco API
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThingDashboardConfig {
    /// Serial number
    pub serial_number: String,
    /// Model code
    pub model_code: Option<String>,
    /// Model name
    pub model_name: Option<String>,
    /// Machine configuration
    pub config: Option<HashMap<String, serde_json::Value>>,
    /// Machine widgets
    #[serde(default)]
    pub widgets: HashMap<String, serde_json::Value>,
}

/// Response for things (machines) from the API
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Thing {
    /// Serial number
    pub serial_number: String,
    /// Model name
    pub model_name: String,
    /// Thing ID
    pub id: String,
}

/// Response from a command sent to the API
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommandResponse {
    /// Command ID
    pub id: String,
    /// Command status
    pub status: String,
}