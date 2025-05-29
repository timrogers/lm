use anyhow::Result;
use log::{debug, error};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};

use crate::auth;
use crate::types::{Machine, MachineCommand, MachineStatus, MachinesResponse};

pub struct LaMarzoccoClient {
    client: reqwest::Client,
    access_token: Option<String>,
    base_url: String,
}

impl Default for LaMarzoccoClient {
    fn default() -> Self {
        Self::new()
    }
}

impl LaMarzoccoClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            access_token: None,
            base_url: "https://lion.lamarzocco.io/api/customer-app".to_string(),
        }
    }

    // Test-specific accessor methods
    #[allow(dead_code)]
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    #[allow(dead_code)]
    pub fn access_token(&self) -> &Option<String> {
        &self.access_token
    }

    // Test-specific constructor for custom base URLs
    #[allow(dead_code)]
    pub fn new_with_base_url(base_url: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            access_token: None,
            base_url,
        }
    }

    pub async fn authenticate(&mut self, username: &str, password: &str) -> Result<()> {
        debug!("Authenticating user: {}", username);

        let token =
            auth::authenticate_with_url(&self.client, &self.base_url, username, password).await?;
        self.access_token = Some(token);

        debug!("Authentication successful");
        Ok(())
    }

    fn get_headers(&self) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        if let Some(token) = &self.access_token {
            let auth_value = format!("Bearer {}", token);
            headers.insert(AUTHORIZATION, HeaderValue::from_str(&auth_value)?);
        } else {
            return Err(anyhow::anyhow!(
                "Not authenticated. Call authenticate() first."
            ));
        }

        Ok(headers)
    }

    pub async fn get_machines(&self) -> Result<Vec<Machine>> {
        debug!("Fetching machines list");

        let url = format!("{}/things", self.base_url);
        let headers = self.get_headers()?;

        let response = self.client.get(&url).headers(headers).send().await?;

        let status = response.status();
        if status.is_success() {
            let response_text = response.text().await?;

            // Try to parse it as a direct array first
            match serde_json::from_str::<Vec<Machine>>(&response_text) {
                Ok(machines) => {
                    debug!("Found {} machines", machines.len());
                    Ok(machines)
                }
                Err(_) => {
                    // If that fails, try parsing as an object with 'things' field
                    match serde_json::from_str::<MachinesResponse>(&response_text) {
                        Ok(machines_response) => {
                            debug!(
                                "Found {} machines (wrapped in 'things')",
                                machines_response.things.len()
                            );
                            Ok(machines_response.things)
                        }
                        Err(e) => {
                            error!("Failed to parse machines response: {}", e);
                            Err(anyhow::anyhow!("Failed to parse machines response: {}", e))
                        }
                    }
                }
            }
        } else {
            let error_text = response.text().await?;
            error!("Failed to fetch machines: {}", error_text);
            Err(anyhow::anyhow!("Failed to fetch machines: {}", error_text))
        }
    }

    pub async fn get_machine_status(&self, serial_number: &str) -> Result<MachineStatus> {
        debug!("Fetching status for machine: {}", serial_number);

        let url = format!("{}/things/{}/dashboard", self.base_url, serial_number);
        let headers = self.get_headers()?;

        let response = self.client.get(&url).headers(headers).send().await?;

        let status = response.status();
        if status.is_success() {
            let response_text = response.text().await?;

            match serde_json::from_str::<MachineStatus>(&response_text) {
                Ok(status) => {
                    debug!("Machine {} status: on={}", serial_number, status.is_on());
                    Ok(status)
                }
                Err(e) => {
                    error!("Failed to parse machine status: {}", e);
                    debug!("Raw response: {}", response_text);
                    Err(anyhow::anyhow!("Failed to parse machine status: {}", e))
                }
            }
        } else {
            let error_text = response.text().await?;
            error!("Failed to fetch machine status: {}", error_text);
            Err(anyhow::anyhow!(
                "Failed to fetch machine status: {}",
                error_text
            ))
        }
    }

    pub async fn turn_on_machine(&self, serial_number: &str) -> Result<()> {
        debug!("Turning on machine: {}", serial_number);
        self.send_machine_command(serial_number, MachineCommand::turn_on())
            .await
    }

    pub async fn turn_off_machine(&self, serial_number: &str) -> Result<()> {
        debug!("Turning off machine: {}", serial_number);
        self.send_machine_command(serial_number, MachineCommand::turn_off())
            .await
    }

    async fn send_machine_command(
        &self,
        serial_number: &str,
        command: MachineCommand,
    ) -> Result<()> {
        let url = format!(
            "{}/things/{}/command/CoffeeMachineChangeMode",
            self.base_url, serial_number
        );
        let headers = self.get_headers()?;

        debug!("Sending command to {}: {:?}", serial_number, command);

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(&command)
            .send()
            .await?;

        if response.status().is_success() {
            debug!("Command sent successfully to machine: {}", serial_number);
            Ok(())
        } else {
            let error_text = response.text().await?;
            error!("Failed to send command to machine: {}", error_text);
            Err(anyhow::anyhow!(
                "Failed to send command to machine: {}",
                error_text
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = LaMarzoccoClient::new();
        assert_eq!(
            client.base_url(),
            "https://lion.lamarzocco.io/api/customer-app"
        );
        assert!(client.access_token().is_none());
    }

    #[test]
    fn test_client_with_custom_base_url() {
        let custom_url = "https://test.example.com".to_string();
        let client = LaMarzoccoClient::new_with_base_url(custom_url.clone());
        assert_eq!(client.base_url(), custom_url);
        assert!(client.access_token().is_none());
    }
}
