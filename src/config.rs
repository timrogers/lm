use crate::error::{Error, Result};
use crate::models::Config;
use dirs::home_dir;
use std::fs;
use std::path::PathBuf;

const CONFIG_FILE_NAME: &str = ".lm.yml";

/// Load configuration from the user's home directory
pub fn load_config() -> Result<Config> {
    let config_path = get_config_path()?;

    if !config_path.exists() {
        return Err(Error::Config(format!(
            "Configuration file not found at {}",
            config_path.display()
        )));
    }

    let config_str = fs::read_to_string(config_path)?;
    let config: Config = serde_yaml::from_str(&config_str)?;

    Ok(config)
}

/// Save configuration to the user's home directory
pub fn save_config(config: &Config) -> Result<()> {
    let config_path = get_config_path()?;
    let config_str = serde_yaml::to_string(config)?;

    fs::write(config_path, config_str)?;

    Ok(())
}

/// Get the path to the configuration file
fn get_config_path() -> Result<PathBuf> {
    let home = home_dir().ok_or_else(|| Error::Config("Could not find home directory".to_string()))?;
    Ok(home.join(CONFIG_FILE_NAME))
}

/// Create initial configuration with provided credentials
pub fn create_config(username: &str, password: &str) -> Result<()> {
    let config = Config {
        username: username.to_string(),
        password: password.to_string(),
        token: None,
    };

    save_config(&config)
}

/// Check if configuration file exists
pub fn config_exists() -> bool {
    match get_config_path() {
        Ok(path) => path.exists(),
        Err(_) => false,
    }
}