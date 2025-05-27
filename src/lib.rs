use anyhow::{Context, Result};
use jsonwebtoken::{decode, DecodingKey, TokenData, Validation};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const BASE_URL: &str = "https://lion.lamarzocco.io/api/customer-app";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Credentials {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    #[serde(rename = "accessToken")]
    pub access_token: String,
    #[serde(rename = "refreshToken")]
    pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct JwtPayload {
    exp: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Machine {
    pub serial_number: String,
    pub model: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MachineStatus {
    pub serial_number: String,
    pub model: String,
    pub name: String,
    pub turned_on: bool,
    pub ready: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandRequest {
    pub mode: String,
}

pub struct LaMarzoccoClient {
    client: Client,
    credentials: Option<Credentials>,
}

impl LaMarzoccoClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            credentials: None,
        }
    }

    #[doc(hidden)]
    pub fn extract_jwt_expiry(token: &str) -> Result<u64> {
        let mut validation = Validation::default();
        validation.insecure_disable_signature_validation();

        let token_data: TokenData<JwtPayload> =
            decode(token, &DecodingKey::from_secret(&[]), &validation)
                .context("Failed to decode JWT")?;

        Ok(token_data.claims.exp)
    }

    pub fn get_config_path() -> Result<PathBuf> {
        let home_dir = dirs::home_dir().context("Unable to find home directory")?;
        Ok(home_dir.join(".lm.yml"))
    }

    pub fn load_credentials(&mut self) -> Result<()> {
        let config_path = Self::get_config_path()?;
        if config_path.exists() {
            let content = fs::read_to_string(&config_path).context("Failed to read config file")?;

            if !content.trim().is_empty() {
                match serde_yaml::from_str(&content) {
                    Ok(creds) => self.credentials = Some(creds),
                    Err(_) => {
                        // Invalid config file, remove it and start fresh
                        let _ = fs::remove_file(&config_path);
                        self.credentials = None;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn save_credentials(&self) -> Result<()> {
        if let Some(ref creds) = self.credentials {
            let config_path = Self::get_config_path()?;
            let content =
                serde_yaml::to_string(creds).context("Failed to serialize credentials")?;
            fs::write(&config_path, content).context("Failed to write config file")?;
        }
        Ok(())
    }

    pub async fn authenticate(&mut self, username: &str, password: &str) -> Result<()> {
        let auth_request = AuthRequest {
            username: username.to_string(),
            password: password.to_string(),
        };

        let response = self
            .client
            .post(&format!("{}/auth/signin", BASE_URL))
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

        let expires_at = Self::extract_jwt_expiry(&auth_response.access_token)
            .context("Failed to extract expiry from access token")?;

        self.credentials = Some(Credentials {
            access_token: auth_response.access_token,
            refresh_token: auth_response.refresh_token,
            expires_at,
        });

        self.save_credentials()?;
        Ok(())
    }

    pub async fn refresh_token_if_needed(&mut self) -> Result<()> {
        if let Some(ref creds) = self.credentials.clone() {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();

            if now + 600 >= creds.expires_at {
                let refresh_request = RefreshRequest {
                    refresh_token: creds.refresh_token.clone(),
                };

                let response = self
                    .client
                    .post(&format!("{}/auth/refreshtoken", BASE_URL))
                    .json(&refresh_request)
                    .send()
                    .await
                    .context("Failed to refresh token")?;

                if !response.status().is_success() {
                    return Err(anyhow::anyhow!(
                        "Token refresh failed: {}",
                        response.status()
                    ));
                }

                let auth_response: AuthResponse = response
                    .json()
                    .await
                    .context("Failed to parse refresh response")?;

                let expires_at = Self::extract_jwt_expiry(&auth_response.access_token)
                    .context("Failed to extract expiry from refreshed access token")?;

                self.credentials = Some(Credentials {
                    access_token: auth_response.access_token,
                    refresh_token: auth_response.refresh_token,
                    expires_at,
                });

                self.save_credentials()?;
            }
        }
        Ok(())
    }

    pub async fn get_machines(&mut self) -> Result<Vec<Machine>> {
        self.refresh_token_if_needed().await?;

        let creds = self.credentials.as_ref().context("Not authenticated")?;

        let response = self
            .client
            .get(&format!("{}/things", BASE_URL))
            .header("Authorization", format!("Bearer {}", creds.access_token))
            .send()
            .await
            .context("Failed to get machines")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Failed to get machines: {}",
                response.status()
            ));
        }

        let things: serde_json::Value = response
            .json()
            .await
            .context("Failed to parse machines response")?;

        let mut machines = Vec::new();
        if let Some(things_array) = things.as_array() {
            for thing in things_array {
                if let (Some(serial), Some(model), Some(name)) = (
                    thing["serialNumber"].as_str(),
                    thing["modelCode"].as_str(),
                    thing["name"].as_str(),
                ) {
                    machines.push(Machine {
                        serial_number: serial.to_string(),
                        model: model.to_string(),
                        name: name.to_string(),
                    });
                }
            }
        }

        Ok(machines)
    }

    pub async fn get_machine_status(&mut self, serial_number: &str) -> Result<MachineStatus> {
        self.refresh_token_if_needed().await?;

        let creds = self.credentials.as_ref().context("Not authenticated")?;

        let response = self
            .client
            .get(&format!("{}/things/{}/dashboard", BASE_URL, serial_number))
            .header("Authorization", format!("Bearer {}", creds.access_token))
            .send()
            .await
            .context("Failed to get machine status")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Failed to get machine status: {}",
                response.status()
            ));
        }

        let dashboard: serde_json::Value = response
            .json()
            .await
            .context("Failed to parse dashboard response")?;

        println!("Dashboard response: {:?}", dashboard);

        let mut turned_on = false;
        let mut ready = false;
        let mut model = String::new();
        let mut name = String::new();

        if let Some(widgets) = dashboard["widgets"].as_array() {
            for widget in widgets {
                if let Some(widget_type) = widget["code"].as_str() {
                    match widget_type {
                        "CMMachineStatus" => {
                            if let Some(status) = widget["output"]["status"].as_str() {
                                turned_on = status != "StandBy" && status != "Off";
                                ready = status == "PoweredOn";
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        if let Some(model_code) = dashboard["modelCode"].as_str() {
            model = model_code.to_string();
        }
        if let Some(thing_name) = dashboard["name"].as_str() {
            name = thing_name.to_string();
        }

        Ok(MachineStatus {
            serial_number: serial_number.to_string(),
            model,
            name,
            turned_on,
            ready,
        })
    }

    pub async fn set_machine_power(&mut self, serial_number: &str, on: bool) -> Result<()> {
        self.refresh_token_if_needed().await?;

        let creds = self.credentials.as_ref().context("Not authenticated")?;

        let mode = if on { "BrewingMode" } else { "StandBy" };
        let command_data = CommandRequest {
            mode: mode.to_string(),
        };

        let response = self
            .client
            .post(&format!(
                "{}/things/{}/command/CoffeeMachineChangeMode",
                BASE_URL, serial_number
            ))
            .header("Authorization", format!("Bearer {}", creds.access_token))
            .json(&command_data)
            .send()
            .await
            .context("Failed to send power command")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Failed to set machine power: {}",
                response.status()
            ));
        }

        Ok(())
    }

    pub fn is_authenticated(&self) -> bool {
        self.credentials.is_some()
    }
}
