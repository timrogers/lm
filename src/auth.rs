use crate::config::{load_config, save_config};
use crate::error::{Error, Result};
use crate::models::{AccessToken, Config, RefreshTokenRequest, SigninRequest};
use reqwest::Client;
use std::time::{SystemTime, UNIX_EPOCH};

const CUSTOMER_APP_URL: &str = "https://lion.lamarzocco.io/api/customer-app";
const TOKEN_TIME_TO_REFRESH: u64 = 600; // 10 minutes before expiration

/// Get an access token, either from the stored token or by authenticating
pub async fn get_access_token(client: &Client) -> Result<String> {
    let mut config = load_config()?;
    
    // Check if token exists and is valid
    if let Some(token) = &config.token {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // If token is expired, get a new one
        if token.expires_at < now {
            let new_token = sign_in(client, &config.username, &config.password).await?;
            config.token = Some(new_token.clone());
            save_config(&config)?;
            return Ok(new_token.access_token);
        }
        
        // If token expires soon, refresh it
        if token.expires_at < now + TOKEN_TIME_TO_REFRESH {
            let refreshed_token = refresh_token(client, &config).await?;
            config.token = Some(refreshed_token.clone());
            save_config(&config)?;
            return Ok(refreshed_token.access_token);
        }
        
        // Token is valid
        return Ok(token.access_token.clone());
    }
    
    // No token, get a new one
    let new_token = sign_in(client, &config.username, &config.password).await?;
    config.token = Some(new_token.clone());
    save_config(&config)?;
    Ok(new_token.access_token)
}

/// Sign in with username and password to get an access token
async fn sign_in(client: &Client, username: &str, password: &str) -> Result<AccessToken> {
    let signin_request = SigninRequest {
        username: username.to_string(),
        password: password.to_string(),
    };
    
    let response = client
        .post(&format!("{}/auth/signin", CUSTOMER_APP_URL))
        .json(&signin_request)
        .send()
        .await?;
    
    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await?;
        if status.as_u16() == 401 {
            return Err(Error::Auth("Invalid username or password".to_string()));
        }
        return Err(Error::Api(format!(
            "Authentication request failed with status code {}: {}",
            status, text
        )));
    }
    
    let token = response.json::<AccessToken>().await?;
    Ok(token)
}

/// Refresh an access token
async fn refresh_token(client: &Client, config: &Config) -> Result<AccessToken> {
    let token = config.token.as_ref().ok_or_else(|| Error::Auth("No token available".to_string()))?;
    
    let refresh_request = RefreshTokenRequest {
        username: config.username.clone(),
        refresh_token: token.refresh_token.clone(),
    };
    
    let response = client
        .post(&format!("{}/auth/refreshtoken", CUSTOMER_APP_URL))
        .json(&refresh_request)
        .send()
        .await?;
    
    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await?;
        if status.as_u16() == 401 {
            return Err(Error::Auth("Invalid refresh token".to_string()));
        }
        return Err(Error::Api(format!(
            "Refresh token request failed with status code {}: {}",
            status, text
        )));
    }
    
    let token = response.json::<AccessToken>().await?;
    Ok(token)
}