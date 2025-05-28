use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use anyhow::Context;

pub type Error = anyhow::Error;
pub type Result<T> = anyhow::Result<T>;

/// Authentication configuration stored in .lm.yml
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub email: String,
    pub password: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
}

impl Config {
    fn config_path() -> Result<PathBuf> {
        let home_dir = dirs::home_dir().context("Could not find home directory")?;
        Ok(home_dir.join(".lm.yml"))
    }

    pub async fn load() -> Result<Config> {
        let config_path = Self::config_path()?;
        
        if !config_path.exists() {
            return Err(anyhow::anyhow!(
                "Configuration file not found at {}. Please create it with your email and password.",
                config_path.display()
            ));
        }

        let content = tokio::fs::read_to_string(&config_path).await
            .context("Failed to read configuration file")?;
        
        let config: Config = serde_yaml::from_str(&content)
            .context("Failed to parse configuration file")?;
        
        Ok(config)
    }

    pub async fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        let content = serde_yaml::to_string(self)
            .context("Failed to serialize configuration")?;
        
        tokio::fs::write(&config_path, content).await
            .context("Failed to write configuration file")?;
        
        Ok(())
    }
}

/// Represents a La Marzocco espresso machine
#[derive(Debug, Serialize, Deserialize)]
pub struct Machine {
    pub serial_number: String,
    pub model: String,
    pub is_on: bool,
    pub is_ready: bool,
}

/// Authentication response from La Marzocco API
#[derive(Debug, Deserialize)]
struct AuthResponse {
    #[serde(rename = "accessToken")]
    access_token: String,
    #[serde(rename = "refreshToken")]
    refresh_token: String,
}

/// Auth request for La Marzocco API
#[derive(Debug, Serialize)]
struct AuthRequest {
    username: String,
    password: String,
}

/// Dashboard response from La Marzocco API containing machine info
#[derive(Debug, Deserialize)]
struct DashboardResponse {
    widgets: Vec<Widget>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Widget {
    code: String,
    output: WidgetOutput,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
#[allow(dead_code)]
enum WidgetOutput {
    MachineStatus {
        status: String,
        #[serde(rename = "availableModes")]
        available_modes: Vec<String>,
        mode: String,
    },
    Thing {
        #[serde(rename = "serialNumber")]
        serial_number: Option<String>,
        #[serde(rename = "modelCode")]
        model_code: Option<String>,
        model: Option<String>,
    },
    Other(serde_json::Value),
}

/// La Marzocco Cloud API client
pub struct LaMarzocoClient {
    client: reqwest::Client,
    config: Config,
    base_url: String,
}

impl LaMarzocoClient {
    pub async fn new(mut config: Config) -> Result<Self> {
        let client = reqwest::Client::new();
        let base_url = "https://gw-lmz.lamarzocco.com".to_string();
        
        // Check if we need to refresh or get new tokens
        let needs_auth = config.access_token.is_none() || 
            config.expires_at.map_or(true, |exp| exp < Utc::now());
        
        if needs_auth {
            Self::authenticate(&client, &base_url, &mut config).await?;
        }

        Ok(Self {
            client,
            config,
            base_url,
        })
    }

    async fn authenticate(
        client: &reqwest::Client,
        base_url: &str,
        config: &mut Config,
    ) -> Result<()> {
        let auth_url = format!("{}/v1/auth/signin", base_url);
        let auth_request = AuthRequest {
            username: config.email.clone(),
            password: config.password.clone(),
        };

        let response = client
            .post(&auth_url)
            .json(&auth_request)
            .send()
            .await
            .context("Failed to send authentication request")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Authentication failed: {}",
                response.status()
            ));
        }

        let auth_response: AuthResponse = response
            .json()
            .await
            .context("Failed to parse authentication response")?;

        config.access_token = Some(auth_response.access_token);
        config.refresh_token = Some(auth_response.refresh_token);
        config.expires_at = Some(Utc::now() + chrono::Duration::hours(1));

        config.save().await?;
        
        Ok(())
    }

    async fn get_auth_header(&self) -> Result<String> {
        let token = self.config.access_token.as_ref()
            .context("No access token available")?;
        Ok(format!("Bearer {}", token))
    }

    pub async fn get_machines(&self) -> Result<Vec<Machine>> {
        // First, get the list of machines from customer endpoint
        let customer_url = format!("{}/api/customer", self.base_url);
        let auth_header = self.get_auth_header().await?;

        let response = self
            .client
            .get(&customer_url)
            .header("Authorization", &auth_header)
            .send()
            .await
            .context("Failed to get customer data")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Failed to get customer data: {}",
                response.status()
            ));
        }

        let customer_data: serde_json::Value = response
            .json()
            .await
            .context("Failed to parse customer response")?;

        // Extract machines from customer data - this is a simplified approach
        // In reality, we'd need to parse the full customer structure
        let mut machines = Vec::new();
        
        // For now, create a mock machine - in reality we'd parse the customer data
        // to get the list of machines and their serial numbers
        if let Some(devices) = customer_data.get("data").and_then(|d| d.get("devices")) {
            if let Some(device_array) = devices.as_array() {
                for device in device_array {
                    if let Some(serial) = device.get("serialNumber").and_then(|s| s.as_str()) {
                        // Get detailed machine status
                        if let Ok(machine) = self.get_machine_status(serial).await {
                            machines.push(machine);
                        }
                    }
                }
            }
        }

        // If no machines found in customer data, return empty list
        // In a real implementation, we'd handle this more gracefully
        Ok(machines)
    }

    async fn get_machine_status(&self, serial_number: &str) -> Result<Machine> {
        let config_url = format!("{}/api/config/{}", self.base_url, serial_number);
        let auth_header = self.get_auth_header().await?;

        let response = self
            .client
            .get(&config_url)
            .header("Authorization", &auth_header)
            .send()
            .await
            .context("Failed to get machine config")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Failed to get machine config: {}",
                response.status()
            ));
        }

        let dashboard: DashboardResponse = response
            .json()
            .await
            .context("Failed to parse dashboard response")?;

        // Parse machine info from widgets
        let mut model = "Unknown".to_string();
        let mut is_on = false;
        let mut is_ready = false;

        for widget in dashboard.widgets {
            match widget.output {
                WidgetOutput::Thing { model_code, model: model_name, .. } => {
                    if let Some(m) = model_name.or(model_code) {
                        model = m;
                    }
                }
                WidgetOutput::MachineStatus { status, mode, .. } => {
                    is_on = mode != "StandBy" && mode != "Off";
                    is_ready = status == "Ready" || status == "StandBy";
                }
                _ => {}
            }
        }

        Ok(Machine {
            serial_number: serial_number.to_string(),
            model,
            is_on,
            is_ready,
        })
    }

    pub async fn set_power(&self, serial_number: &str, enabled: bool) -> Result<()> {
        let command_url = format!("{}/api/device/{}/command", self.base_url, serial_number);
        let auth_header = self.get_auth_header().await?;

        let command = if enabled { "ON" } else { "OFF" };
        let payload = serde_json::json!({
            "command": "set_power",
            "parameters": {
                "enabled": enabled
            }
        });

        let response = self
            .client
            .post(&command_url)
            .header("Authorization", &auth_header)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .context("Failed to send power command")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Failed to set power {}: {}",
                command,
                response.status()
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_serialization_works() {
        let config = Config {
            email: "test@example.com".to_string(),
            password: "password".to_string(),
            access_token: Some("token".to_string()),
            refresh_token: Some("refresh".to_string()),
            expires_at: Some(Utc::now()),
        };

        let yaml = serde_yaml::to_string(&config).unwrap();
        let deserialized: Config = serde_yaml::from_str(&yaml).unwrap();
        
        assert_eq!(config.email, deserialized.email);
        assert_eq!(config.password, deserialized.password);
        assert_eq!(config.access_token, deserialized.access_token);
        assert_eq!(config.refresh_token, deserialized.refresh_token);
    }
}
