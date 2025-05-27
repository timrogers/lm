use crate::error::{LaMarzoccoError, Result};
use crate::models::{AuthResponse, Credentials};
use chrono::{Duration, Utc};
use dirs::home_dir;
use serde_json::json;
use std::fs;
use std::path::PathBuf;

const CUSTOMER_APP_URL: &str = "https://lion.lamarzocco.io/api/customer-app";
const CONFIG_FILENAME: &str = ".lm.yml";
const TOKEN_REFRESH_WINDOW_SECS: i64 = 600; // 10 minutes before token expires

/// Handles authentication and token storage for La Marzocco API
pub struct Auth {
    credentials: Option<Credentials>,
    client: reqwest::Client,
}

impl Auth {
    /// Create a new Auth instance
    pub fn new() -> Self {
        Auth {
            credentials: None,
            client: reqwest::Client::new(),
        }
    }

    /// Get the path to the config file
    pub fn get_config_path() -> Result<PathBuf> {
        let home = home_dir().ok_or_else(|| {
            LaMarzoccoError::ConfigError("Could not determine home directory".to_string())
        })?;
        Ok(home.join(CONFIG_FILENAME))
    }

    /// Load credentials from config file
    pub fn load_credentials(&mut self) -> Result<&Credentials> {
        // If we already have credentials, return them
        if let Some(ref credentials) = self.credentials {
            return Ok(credentials);
        }

        // Load from config file
        let config_path = Self::get_config_path()?;
        if !config_path.exists() {
            return Err(LaMarzoccoError::ConfigError(
                "Config file not found. Please login first.".to_string(),
            ));
        }

        let config_data = fs::read_to_string(&config_path)?;
        let credentials: Credentials = serde_yaml::from_str(&config_data)?;
        self.credentials = Some(credentials);

        Ok(self.credentials.as_ref().unwrap())
    }

    /// Save credentials to config file
    pub fn save_credentials(&self, credentials: &Credentials) -> Result<()> {
        let config_path = Self::get_config_path()?;
        let config_data = serde_yaml::to_string(credentials)?;
        fs::write(config_path, config_data)?;
        Ok(())
    }

    /// Authenticate and get a token
    pub async fn authenticate(&mut self, username: &str, password: &str) -> Result<()> {
        let auth_url = format!("{}/auth/signin", CUSTOMER_APP_URL);

        let response = self
            .client
            .post(&auth_url)
            .json(&json!({
                "username": username,
                "password": password,
            }))
            .send()
            .await?;

        if response.status().is_success() {
            let auth_response: AuthResponse = response.json().await?;
            // Calculate expiration time
            let expires_at = Utc::now() + Duration::seconds(auth_response.expires_in);

            let credentials = Credentials {
                username: username.to_string(),
                access_token: auth_response.access_token,
                refresh_token: auth_response.refresh_token,
                expires_at,
            };

            self.save_credentials(&credentials)?;
            self.credentials = Some(credentials);
            Ok(())
        } else {
            Err(LaMarzoccoError::ApiError {
                status_code: response.status().as_u16(),
                message: format!("Authentication failed: {}", response.status()),
            })
        }
    }

    /// Get a valid access token, refreshing if necessary
    pub async fn get_access_token(&mut self) -> Result<String> {
        let credentials = self.load_credentials()?;
        
        // Check if token is expired or will expire soon
        if Utc::now() + Duration::seconds(TOKEN_REFRESH_WINDOW_SECS) >= credentials.expires_at {
            self.refresh_token().await?;
            return Ok(self.credentials.as_ref().unwrap().access_token.clone());
        }
        
        Ok(credentials.access_token.clone())
    }

    /// Refresh the token using the refresh token
    async fn refresh_token(&mut self) -> Result<()> {
        let credentials = self.load_credentials()?.clone(); // Clone to avoid borrow conflicts
        let refresh_url = format!("{}/auth/refreshtoken", CUSTOMER_APP_URL);

        let response = self
            .client
            .post(&refresh_url)
            .json(&json!({
                "username": credentials.username,
                "refresh_token": credentials.refresh_token,
            }))
            .send()
            .await?;

        if response.status().is_success() {
            let auth_response: AuthResponse = response.json().await?;
            // Calculate expiration time
            let expires_at = Utc::now() + Duration::seconds(auth_response.expires_in);

            let new_credentials = Credentials {
                username: credentials.username.clone(),
                access_token: auth_response.access_token,
                refresh_token: auth_response.refresh_token,
                expires_at,
            };

            self.save_credentials(&new_credentials)?;
            self.credentials = Some(new_credentials);
            Ok(())
        } else {
            Err(LaMarzoccoError::ApiError {
                status_code: response.status().as_u16(),
                message: format!("Token refresh failed: {}", response.status()),
            })
        }
    }
}