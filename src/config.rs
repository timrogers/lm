use anyhow::{Context, Result};
use dirs::home_dir;
use log::{debug, warn};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::installation_key::InstallationKey;
use crate::types::Credentials;

/// Configuration data stored in ~/.lm.yml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub username: String,
    pub access_token: String,
    pub refresh_token: String,
    /// Installation key for new authentication system
    #[serde(skip_serializing_if = "Option::is_none")]
    pub installation_key: Option<InstallationKey>,
}

impl From<&Credentials> for Config {
    fn from(credentials: &Credentials) -> Self {
        Self {
            username: credentials.username.clone(),
            access_token: credentials.access_token.clone(),
            refresh_token: credentials.refresh_token.clone(),
            installation_key: credentials.installation_key.clone(),
        }
    }
}

impl From<Config> for Credentials {
    fn from(config: Config) -> Self {
        Self {
            username: config.username,
            access_token: config.access_token,
            refresh_token: config.refresh_token,
            installation_key: config.installation_key,
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

    // First attempt to parse as full Config. If required fields are missing, return a clearer error.
    match serde_yaml::from_str::<Config>(&content) {
        Ok(config) => {
            debug!("Loaded configuration for user: {}", config.username);
            return Ok(config);
        }
        Err(_) => {
            // If the file exists but isn't a full config (e.g., only installation_key), surface a friendly error
            return Err(anyhow::anyhow!(
                "Configuration incomplete. Please run 'lm login' first."
            ));
        }
    }
}

/// Save configuration to ~/.lm.yml
pub fn save_config(config: &Config) -> Result<()> {
    let config_path = get_config_path()?;

    let content = serde_yaml::to_string(config).context("Failed to serialize configuration")?;

    fs::write(&config_path, content)
        .with_context(|| format!("Failed to write config file: {}", config_path.display()))?;

    debug!("Saved configuration for user: {}", config.username);
    Ok(())
}

/// Load only the installation key from the main config file if present
pub fn load_installation_key_partial() -> Result<InstallationKey> {
    let path = get_config_path()?;
    if !path.exists() {
        return Err(anyhow::anyhow!("Installation key not found"));
    }

    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read config file: {}", path.display()))?;

    let mut value: serde_yaml::Value = serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse config file: {}", path.display()))?;

    if let Some(install_val) = value.get_mut("installation_key") {
        let key: InstallationKey = serde_yaml::from_value(install_val.clone())
            .context("Failed to parse installation_key from config")?;
        debug!(
            "Loaded installation key from main config: {}",
            key.installation_id
        );
        Ok(key)
    } else {
        Err(anyhow::anyhow!("Installation key not found"))
    }
}

/// Save/update only the installation key inside the main config file
pub fn save_installation_key_partial(key: &InstallationKey) -> Result<()> {
    let path = get_config_path()?;
    let mut root = if path.exists() {
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;
        serde_yaml::from_str::<serde_yaml::Value>(&content)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?
    } else {
        serde_yaml::Value::Mapping(serde_yaml::Mapping::new())
    };

    let key_val = serde_yaml::to_value(key).context("Failed to serialize installation key")?;

    if let serde_yaml::Value::Mapping(ref mut map) = root {
        map.insert(
            serde_yaml::Value::String("installation_key".to_string()),
            key_val,
        );
    } else {
        // If the root isn't a mapping, replace it with a mapping
        let mut map = serde_yaml::Mapping::new();
        map.insert(
            serde_yaml::Value::String("installation_key".to_string()),
            key_val,
        );
        root = serde_yaml::Value::Mapping(map);
    }

    let content = serde_yaml::to_string(&root).context("Failed to serialize YAML")?;
    fs::write(&path, content)
        .with_context(|| format!("Failed to write config file: {}", path.display()))?;
    debug!(
        "Saved installation key to main config: {}",
        key.installation_id
    );
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
            installation_key: None,
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
            installation_key: None,
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
