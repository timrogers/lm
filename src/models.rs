use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

const TOKEN_EXPIRATION: u64 = 3600; // 1 hour

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccessToken {
    pub access_token: String,
    pub refresh_token: String,
    #[serde(default = "default_expires_at")]
    pub expires_at: u64,
}

fn default_expires_at() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + TOKEN_EXPIRATION
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SigninRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshTokenRequest {
    pub username: String,
    pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub username: String,
    pub password: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<AccessToken>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Machine {
    pub serial_number: String,
    pub name: String,
    pub model_name: String,
    pub firmware_version: String,
    pub status: MachineStatus,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum MachineStatus {
    StandBy,
    PoweredOn,
    Brewing,
    Off,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MachineState {
    pub status: MachineStatus,
    pub is_ready: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Thing {
    #[serde(rename = "serialNumber")]
    pub serial_number: String,
    pub name: String,
    #[serde(rename = "modelName")]
    pub model_name: String,
    #[serde(rename = "modelCode")]
    pub model_code: String,
    #[serde(rename = "deviceType")]
    pub device_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ThingDashboardConfig {
    pub widgets: Vec<Widget>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Widget {
    pub code: String,
    pub output: serde_json::Value,
}