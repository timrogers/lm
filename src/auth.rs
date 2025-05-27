use crate::error::{LaMarzoccoError, Result};
use crate::models::{AuthResponse, Credentials};
use chrono::{Duration, Utc};
use dirs::home_dir;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;

const CUSTOMER_APP_URL: &str = "https://lion.lamarzocco.io/api/customer-app";
const CONFIG_FILENAME: &str = ".lm.yml";
const TOKEN_REFRESH_WINDOW_SECS: i64 = 600; // 10 minutes before token expires

#[derive(Debug, Serialize, Deserialize)]
struct TokenClaims {
    exp: i64,
    // Other fields in the token are not needed for our use case
}

/// Token refresh callback function
pub type TokenRefreshCallback = Box<dyn FnMut(&Credentials) -> Result<()> + Send>;

/// Handles authentication and token storage for La Marzocco API
pub struct Auth {
    credentials: Option<Credentials>,
    client: reqwest::Client,
    refresh_callback: Option<TokenRefreshCallback>,
}

impl Auth {
    /// Create a new Auth instance
    pub fn new() -> Self {
        Auth {
            credentials: None,
            client: reqwest::Client::new(),
            refresh_callback: None,
        }
    }

    /// Set a callback for token refresh events
    pub fn set_refresh_callback(&mut self, callback: TokenRefreshCallback) {
        self.refresh_callback = Some(callback);
    }

    /// Extract expiry from access token
    pub fn extract_token_expiry(token: &str) -> Result<chrono::DateTime<Utc>> {
        // Token may not have the standard 3 parts, so we'll handle errors gracefully
        let validation = Validation::default();
        
        // JWT tokens don't need validation just to extract the expiry
        match decode::<TokenClaims>(
            token,
            &DecodingKey::from_secret(&[]), // dummy key since we're not validating signature
            &validation,
        ) {
            Ok(token_data) => {
                let exp_timestamp = token_data.claims.exp;
                Ok(chrono::DateTime::<Utc>::from_timestamp(exp_timestamp, 0)
                    .ok_or_else(|| LaMarzoccoError::ConfigError("Invalid token expiry".to_string()))?)
            }
            Err(_) => {
                // If we can't extract the expiry, use a default expiry of 1 hour from now
                Ok(Utc::now() + Duration::hours(1))
            }
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

        let config_data = std::fs::read_to_string(&config_path)?;
        let credentials: Credentials = serde_yaml::from_str(&config_data)?;
        self.credentials = Some(credentials);

        Ok(self.credentials.as_ref().unwrap())
    }

    /// Authenticate and get a token
    pub async fn authenticate(&mut self, username: &str, password: &str) -> Result<Credentials> {
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
            
            // Extract expiry time from token
            let expires_at = Self::extract_token_expiry(&auth_response.access_token)?;

            let credentials = Credentials {
                username: username.to_string(),
                access_token: auth_response.access_token,
                refresh_token: auth_response.refresh_token,
                expires_at,
            };

            self.credentials = Some(credentials.clone());
            Ok(credentials)
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
            let refreshed_credentials = self.refresh_token().await?;
            
            // Notify callback if provided
            if let Some(ref mut callback) = self.refresh_callback {
                callback(&refreshed_credentials)?;
            }
            
            return Ok(refreshed_credentials.access_token.clone());
        }
        
        Ok(credentials.access_token.clone())
    }

    /// Refresh the token using the refresh token
    async fn refresh_token(&mut self) -> Result<Credentials> {
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
            
            // Extract expiry time from token
            let expires_at = Self::extract_token_expiry(&auth_response.access_token)?;

            let new_credentials = Credentials {
                username: credentials.username,
                access_token: auth_response.access_token,
                refresh_token: auth_response.refresh_token,
                expires_at,
            };

            self.credentials = Some(new_credentials.clone());
            Ok(new_credentials)
        } else {
            Err(LaMarzoccoError::ApiError {
                status_code: response.status().as_u16(),
                message: format!("Token refresh failed: {}", response.status()),
            })
        }
    }
}