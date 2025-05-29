use anyhow::{Context, Result};
use dirs::home_dir;
use log::{debug, warn};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::types::Credentials;

/// Configuration data stored in ~/.lm.yml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub username: String,
    pub access_token: String,
    pub refresh_token: String,
}

impl From<&Credentials> for Config {
    fn from(credentials: &Credentials) -> Self {
        Self {
            username: credentials.username.clone(),
            access_token: credentials.access_token.clone(),
            refresh_token: credentials.refresh_token.clone(),
        }
    }
}

impl From<Config> for Credentials {
    fn from(config: Config) -> Self {
        Self {
            username: config.username,
            access_token: config.access_token,
            refresh_token: config.refresh_token,
        }
    }
}

/// Get the path to the configuration file (~/.lm.yml)
pub fn get_config_path() -> Result<PathBuf> {
    let home = home_dir().context("Failed to determine home directory")?;
    Ok(home.join(".lm.yml"))
}

/// Load configuration from ~/.lm.yml
pub fn load_config() -> Result<Config> {
    let config_path = get_config_path()?;
    
    if !config_path.exists() {
        return Err(anyhow::anyhow!(
            "Configuration file not found. Please run 'lm login' first."
        ));
    }

    let content = fs::read_to_string(&config_path)
        .with_context(|| format!("Failed to read config file: {}", config_path.display()))?;

    let config: Config = serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse config file: {}", config_path.display()))?;

    debug!("Loaded configuration for user: {}", config.username);
    Ok(config)
}

/// Save configuration to ~/.lm.yml
pub fn save_config(config: &Config) -> Result<()> {
    let config_path = get_config_path()?;
    
    let content = serde_yaml::to_string(config)
        .context("Failed to serialize configuration")?;

    fs::write(&config_path, content)
        .with_context(|| format!("Failed to write config file: {}", config_path.display()))?;

    debug!("Saved configuration for user: {}", config.username);
    Ok(())
}

/// Clear the configuration file (logout)
pub fn clear_config() -> Result<()> {
    let config_path = get_config_path()?;
    
    if config_path.exists() {
        fs::remove_file(&config_path)
            .with_context(|| format!("Failed to remove config file: {}", config_path.display()))?;
        debug!("Configuration file cleared");
    } else {
        warn!("Configuration file does not exist, nothing to clear");
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_conversion() {
        let credentials = Credentials {
            username: "test@example.com".to_string(),
            access_token: "access123".to_string(),
            refresh_token: "refresh456".to_string(),
        };

        let config = Config::from(&credentials);
        assert_eq!(config.username, "test@example.com");
        assert_eq!(config.access_token, "access123");
        assert_eq!(config.refresh_token, "refresh456");

        let back_to_credentials = Credentials::from(config);
        assert_eq!(back_to_credentials.username, credentials.username);
        assert_eq!(back_to_credentials.access_token, credentials.access_token);
        assert_eq!(back_to_credentials.refresh_token, credentials.refresh_token);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config {
            username: "test@example.com".to_string(),
            access_token: "access123".to_string(),
            refresh_token: "refresh456".to_string(),
        };

        let yaml = serde_yaml::to_string(&config).unwrap();
        assert!(yaml.contains("username: test@example.com"));
        assert!(yaml.contains("access_token: access123"));
        assert!(yaml.contains("refresh_token: refresh456"));

        let parsed: Config = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(parsed.username, config.username);
        assert_eq!(parsed.access_token, config.access_token);
        assert_eq!(parsed.refresh_token, config.refresh_token);
    }
}