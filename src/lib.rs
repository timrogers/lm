use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde::{Deserialize, Serialize};
use thiserror::Error;

// Constants
const CUSTOMER_APP_URL: &str = "https://lion.lamarzocco.io/api/customer-app";
const TOKEN_EXPIRATION: u64 = 3600; // 1 hour in seconds
const TOKEN_TIME_TO_REFRESH: u64 = 600; // 10 minutes in seconds

// Errors
#[derive(Error, Debug)]
pub enum LmError {
    #[error("Authentication failed: {0}")]
    AuthError(String),

    #[error("API request failed: {0}")]
    RequestError(String),

    #[error("Config error: {0}")]
    ConfigError(String),

    #[error("HTTP error: {status_code} - {message}")]
    HttpError { status_code: u16, message: String },

    #[error("{0}")]
    Other(String),
}

// Authentication models
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AccessToken {
    #[serde(rename = "accessToken")]
    pub access_token: String,
    #[serde(rename = "refreshToken")]
    pub refresh_token: String,
    pub expires_at: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct SigninTokenRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct RefreshTokenRequest {
    pub username: String,
    #[serde(rename = "refreshToken")]
    pub refresh_token: String,
}

// Machine models
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Machine {
    #[serde(rename = "serialNumber")]
    pub serial_number: String,
    pub name: String,
    #[serde(rename = "locationName")]
    pub location_name: Option<String>,
    #[serde(rename = "modelName")]
    pub model_name: String,
    pub status: MachineStatus,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum MachineStatus {
    #[serde(rename = "PoweredOn")]
    PoweredOn,
    #[serde(rename = "StandBy")]
    Standby,
    #[serde(rename = "Brewing")]
    Brewing,
    #[serde(rename = "Off")]
    Off,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommandResponse {
    pub id: String,
    pub status: String,
}

// Configuration model
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub username: String,
    pub password: String,
    pub token: Option<AccessToken>,
}

// Client
pub struct LaMarzoccoClient {
    client: reqwest::Client,
    config: Config,
}

impl LaMarzoccoClient {
    /// Create a new La Marzocco client
    pub async fn new(username: String, password: String) -> Result<Self, LmError> {
        let config = Config {
            username,
            password,
            token: None,
        };

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| LmError::Other(e.to_string()))?;

        Ok(Self { client, config })
    }

    /// Load client from saved configuration
    pub async fn from_config() -> Result<Self, LmError> {
        let config = load_config()?;

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| LmError::Other(e.to_string()))?;

        Ok(Self { client, config })
    }

    /// Get a valid access token
    async fn get_access_token(&mut self) -> Result<String, LmError> {
        match &self.config.token {
            Some(token) => {
                let current_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|e| LmError::Other(e.to_string()))?
                    .as_secs();

                if token.expires_at < current_time {
                    self.sign_in().await?;
                } else if token.expires_at < current_time + TOKEN_TIME_TO_REFRESH {
                    self.refresh_token().await?;
                }
            }
            None => {
                self.sign_in().await?;
            }
        }

        // Save config after token update
        save_config(&self.config)?;

        Ok(self.config.token.as_ref().unwrap().access_token.clone())
    }

    /// Sign in and get a new access token
    async fn sign_in(&mut self) -> Result<(), LmError> {
        let signin_request = SigninTokenRequest {
            username: self.config.username.clone(),
            password: self.config.password.clone(),
        };

        let response = self
            .client
            .post(&format!("{}/auth/signin", CUSTOMER_APP_URL))
            .json(&signin_request)
            .send()
            .await
            .map_err(|e| LmError::RequestError(e.to_string()))?;

        if response.status().is_success() {
            let mut token: AccessToken = response
                .json()
                .await
                .map_err(|e| LmError::RequestError(e.to_string()))?;

            // Set token expiration
            let current_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|e| LmError::Other(e.to_string()))?
                .as_secs();
            token.expires_at = current_time + TOKEN_EXPIRATION;

            self.config.token = Some(token);
            Ok(())
        } else if response.status().as_u16() == 401 {
            Err(LmError::AuthError(
                "Invalid username or password".to_string(),
            ))
        } else {
            let status = response.status().as_u16();
            let message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(LmError::HttpError {
                status_code: status,
                message,
            })
        }
    }

    /// Refresh an existing token
    async fn refresh_token(&mut self) -> Result<(), LmError> {
        if self.config.token.is_none() {
            return self.sign_in().await;
        }

        let refresh_token = self.config.token.as_ref().unwrap().refresh_token.clone();

        let refresh_request = RefreshTokenRequest {
            username: self.config.username.clone(),
            refresh_token,
        };

        let response = self
            .client
            .post(&format!("{}/auth/refreshtoken", CUSTOMER_APP_URL))
            .json(&refresh_request)
            .send()
            .await
            .map_err(|e| LmError::RequestError(e.to_string()))?;

        if response.status().is_success() {
            let mut token: AccessToken = response
                .json()
                .await
                .map_err(|e| LmError::RequestError(e.to_string()))?;

            // Set token expiration
            let current_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|e| LmError::Other(e.to_string()))?
                .as_secs();
            token.expires_at = current_time + TOKEN_EXPIRATION;

            self.config.token = Some(token);
            Ok(())
        } else if response.status().as_u16() == 401 {
            // If refresh token is invalid, try logging in again
            self.sign_in().await
        } else {
            let status = response.status().as_u16();
            let message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(LmError::HttpError {
                status_code: status,
                message,
            })
        }
    }

    /// Make a REST API call
    async fn rest_api_call<T: serde::de::DeserializeOwned>(
        &mut self,
        url: &str,
        method: reqwest::Method,
        data: Option<serde_json::Value>,
    ) -> Result<T, LmError> {
        let access_token = self.get_access_token().await?;
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", access_token))
                .map_err(|e| LmError::Other(e.to_string()))?,
        );

        let mut request = self.client.request(method, url).headers(headers);
        if let Some(json_data) = data {
            request = request.json(&json_data);
        }

        let response = request
            .send()
            .await
            .map_err(|e| LmError::RequestError(e.to_string()))?;

        if response.status().is_success() {
            response
                .json()
                .await
                .map_err(|e| LmError::RequestError(e.to_string()))
        } else if response.status().as_u16() == 401 {
            Err(LmError::AuthError("Authentication failed".to_string()))
        } else {
            let status = response.status().as_u16();
            let message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(LmError::HttpError {
                status_code: status,
                message,
            })
        }
    }

    /// List all machines (things)
    pub async fn list_machines(&mut self) -> Result<Vec<Machine>, LmError> {
        let url = format!("{}/things", CUSTOMER_APP_URL);
        self.rest_api_call::<Vec<Machine>>(&url, reqwest::Method::GET, None)
            .await
    }

    /// Turn machine power on
    pub async fn turn_on(&mut self, serial_number: &str) -> Result<CommandResponse, LmError> {
        let url = format!(
            "{}/things/{}/command/CoffeeMachineChangeMode",
            CUSTOMER_APP_URL, serial_number
        );
        let data = serde_json::json!({
            "mode": "BrewingMode"
        });

        let response_array: Vec<CommandResponse> = self
            .rest_api_call(&url, reqwest::Method::POST, Some(data))
            .await?;
        if response_array.is_empty() {
            return Err(LmError::Other("Empty response from server".to_string()));
        }
        Ok(response_array[0].clone())
    }

    /// Turn machine power off
    pub async fn turn_off(&mut self, serial_number: &str) -> Result<CommandResponse, LmError> {
        let url = format!(
            "{}/things/{}/command/CoffeeMachineChangeMode",
            CUSTOMER_APP_URL, serial_number
        );
        let data = serde_json::json!({
            "mode": "StandBy"
        });

        let response_array: Vec<CommandResponse> = self
            .rest_api_call(&url, reqwest::Method::POST, Some(data))
            .await?;
        if response_array.is_empty() {
            return Err(LmError::Other("Empty response from server".to_string()));
        }
        Ok(response_array[0].clone())
    }
}

// Configuration functions
fn get_config_path() -> Result<PathBuf, LmError> {
    let home_dir = dirs::home_dir()
        .ok_or_else(|| LmError::ConfigError("Could not find home directory".to_string()))?;
    Ok(home_dir.join(".lm.yml"))
}

pub fn save_config(config: &Config) -> Result<(), LmError> {
    let config_path = get_config_path()?;
    let yaml = serde_yaml::to_string(config).map_err(|e| LmError::ConfigError(e.to_string()))?;
    fs::write(&config_path, yaml).map_err(|e| LmError::ConfigError(e.to_string()))?;
    Ok(())
}

pub fn load_config() -> Result<Config, LmError> {
    let config_path = get_config_path()?;
    if !config_path.exists() {
        return Err(LmError::ConfigError(
            "Config file not found, please run 'lm login' first".to_string(),
        ));
    }

    let yaml = fs::read_to_string(&config_path).map_err(|e| LmError::ConfigError(e.to_string()))?;
    serde_yaml::from_str(&yaml).map_err(|e| LmError::ConfigError(e.to_string()))
}

pub fn create_initial_config(username: String, password: String) -> Result<(), LmError> {
    let config = Config {
        username,
        password,
        token: None,
    };
    save_config(&config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
