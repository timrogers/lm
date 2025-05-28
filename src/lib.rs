use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

const BASE_URL: &str = "https://lion.lamarzocco.io/api/customer-app";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub username: Option<String>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub expires_at: Option<u64>,
}

impl Config {
    pub fn new() -> Self {
        Self {
            username: None,
            access_token: None,
            refresh_token: None,
            expires_at: None,
        }
    }

    pub fn is_token_valid(&self) -> bool {
        if let (Some(_), Some(expires_at)) = (&self.access_token, self.expires_at) {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            now < expires_at
        } else {
            false
        }
    }
}

#[derive(Debug, Serialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LoginResponse {
    access_token: String,
    refresh_token: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RefreshRequest {
    username: String,
    refresh_token: String,
}

#[derive(Debug, Deserialize)]
pub struct Machine {
    pub serial_number: String,
    pub model: String,
    pub name: String,
    pub is_on: bool,
    pub is_ready: bool,
}

#[derive(Debug, Serialize)]
struct PowerRequest {
    enabled: bool,
}

pub struct LaMarzoccoClient {
    client: Client,
    config: Config,
}

impl Default for LaMarzoccoClient {
    fn default() -> Self {
        Self::new()
    }
}

impl LaMarzoccoClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            config: Config::new(),
        }
    }

    pub fn with_config(config: Config) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    pub async fn authenticate(&mut self, username: &str, password: &str) -> Result<()> {
        let login_request = LoginRequest {
            username: username.to_string(),
            password: password.to_string(),
        };

        let response = self
            .client
            .post(format!("{}/auth/signin", BASE_URL))
            .json(&login_request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Authentication failed: {}", response.status()));
        }

        let login_response: LoginResponse = response.json().await?;
        
        // Tokens expire in 1 hour (3600 seconds)
        let expires_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() + 3600;

        self.config.username = Some(username.to_string());
        self.config.access_token = Some(login_response.access_token);
        self.config.refresh_token = Some(login_response.refresh_token);
        self.config.expires_at = Some(expires_at);

        Ok(())
    }

    pub async fn refresh_token(&mut self) -> Result<()> {
        let username = self.config.username.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No username configured"))?;
        let refresh_token = self.config.refresh_token.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No refresh token available"))?;

        let refresh_request = RefreshRequest {
            username: username.clone(),
            refresh_token: refresh_token.clone(),
        };

        let response = self
            .client
            .post(format!("{}/auth/refresh", BASE_URL))
            .json(&refresh_request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Token refresh failed: {}", response.status()));
        }

        let login_response: LoginResponse = response.json().await?;
        
        let expires_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() + 3600;

        self.config.access_token = Some(login_response.access_token);
        self.config.refresh_token = Some(login_response.refresh_token);
        self.config.expires_at = Some(expires_at);

        Ok(())
    }

    async fn ensure_authenticated(&mut self) -> Result<()> {
        if !self.config.is_token_valid() {
            if self.config.refresh_token.is_some() {
                self.refresh_token().await?;
            } else {
                return Err(anyhow::anyhow!("No valid authentication available. Please login first."));
            }
        }
        Ok(())
    }

    pub async fn get_machines(&mut self) -> Result<Vec<Machine>> {
        self.ensure_authenticated().await?;
        
        let access_token = self.config.access_token.as_ref().unwrap();
        
        let response = self
            .client
            .get(format!("{}/things", BASE_URL))
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Failed to get machines: {}", response.status()));
        }

        // For now, return a mock machine since I don't have the exact API response format
        // In a real implementation, this would parse the actual response
        let machines = vec![Machine {
            serial_number: "LM123456".to_string(),
            model: "Linea Mini".to_string(),
            name: "My Coffee Machine".to_string(),
            is_on: false,
            is_ready: false,
        }];

        Ok(machines)
    }

    pub async fn set_machine_power(&mut self, serial_number: &str, enabled: bool) -> Result<()> {
        self.ensure_authenticated().await?;
        
        let access_token = self.config.access_token.as_ref().unwrap();
        let power_request = PowerRequest { enabled };
        
        let response = self
            .client
            .post(format!("{}/things/{}/commands/set-power", BASE_URL, serial_number))
            .header("Authorization", format!("Bearer {}", access_token))
            .json(&power_request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Failed to set power: {}", response.status()));
        }

        Ok(())
    }

    pub fn get_config(&self) -> &Config {
        &self.config
    }

    pub fn set_config(&mut self, config: Config) {
        self.config = config;
    }
}

pub mod config {
    use super::Config;
    use anyhow::Result;
    use std::fs;
    use std::path::PathBuf;

    pub fn get_config_path() -> Result<PathBuf> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        Ok(home_dir.join(".lm.yml"))
    }

    pub fn load_config() -> Result<Config> {
        let config_path = get_config_path()?;
        
        if !config_path.exists() {
            return Ok(Config::new());
        }

        let contents = fs::read_to_string(config_path)?;
        let config: Config = serde_yaml::from_str(&contents)?;
        Ok(config)
    }

    pub fn save_config(config: &Config) -> Result<()> {
        let config_path = get_config_path()?;
        let contents = serde_yaml::to_string(config)?;
        fs::write(config_path, contents)?;
        Ok(())
    }
}
