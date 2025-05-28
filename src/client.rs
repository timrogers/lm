use crate::auth::get_access_token;
use crate::error::{Error, Result};
use crate::models::{Machine, MachineStatus, Thing, ThingDashboardConfig};
use reqwest::{Client, StatusCode};
use serde::de::DeserializeOwned;
use serde_json::Value;

const CUSTOMER_APP_URL: &str = "https://lion.lamarzocco.io/api/customer-app";

pub struct LaMarzoccoClient {
    client: Client,
}

impl LaMarzoccoClient {
    /// Create a new La Marzocco API client
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    /// Make a GET request to the La Marzocco API
    async fn get<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        let token = get_access_token(&self.client).await?;
        
        let response = self.client
            .get(url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await?;
            if status == StatusCode::UNAUTHORIZED {
                return Err(Error::Auth("Authentication failed".to_string()));
            }
            return Err(Error::Api(format!(
                "Request to {} failed with status code {}: {}",
                url, status, text
            )));
        }
        
        let result = response.json::<T>().await?;
        Ok(result)
    }

    /// Make a POST request to the La Marzocco API
    async fn post<T: DeserializeOwned>(&self, url: &str, data: Option<Value>) -> Result<T> {
        let token = get_access_token(&self.client).await?;
        
        let mut request = self.client
            .post(url)
            .header("Authorization", format!("Bearer {}", token));
            
        if let Some(json_data) = data {
            request = request.json(&json_data);
        }
        
        let response = request.send().await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await?;
            if status == StatusCode::UNAUTHORIZED {
                return Err(Error::Auth("Authentication failed".to_string()));
            }
            return Err(Error::Api(format!(
                "Request to {} failed with status code {}: {}",
                url, status, text
            )));
        }
        
        let result = response.json::<T>().await?;
        Ok(result)
    }

    /// List all machines
    pub async fn list_machines(&self) -> Result<Vec<Machine>> {
        let url = format!("{}/things", CUSTOMER_APP_URL);
        let things: Vec<Thing> = self.get(&url).await?;
        
        // Filter for coffee machines only
        let machines = things
            .into_iter()
            .filter(|thing| thing.device_type == "CoffeeMachine")
            .map(|thing| Machine {
                serial_number: thing.serial_number,
                name: thing.name,
                model_name: thing.model_name,
                firmware_version: "Unknown".to_string(),
                status: MachineStatus::Off,
            })
            .collect();
        
        Ok(machines)
    }

    /// Get machine status
    pub async fn get_machine_status(&self, serial_number: &str) -> Result<Machine> {
        // First, get basic machine info
        let machines = self.list_machines().await?;
        let mut machine = machines
            .into_iter()
            .find(|m| m.serial_number == serial_number)
            .ok_or_else(|| Error::Api(format!("Machine with serial number {} not found", serial_number)))?;
        
        // Then, get the detailed dashboard info
        let url = format!("{}/things/{}/dashboard", CUSTOMER_APP_URL, serial_number);
        let dashboard: ThingDashboardConfig = self.get(&url).await?;
        
        // Look for the machine status widget
        if let Some(status_widget) = dashboard.widgets.iter().find(|w| w.code == "CMMachineStatus") {
            if let Some(status_str) = status_widget.output.get("status").and_then(|s| s.as_str()) {
                machine.status = match status_str {
                    "StandBy" => MachineStatus::StandBy,
                    "PoweredOn" => MachineStatus::PoweredOn,
                    "Brewing" => MachineStatus::Brewing,
                    _ => MachineStatus::Off,
                };
            }
            
            // Get firmware version if available
            if let Some(fw_version) = status_widget.output.get("firmwareVersion").and_then(|v| v.as_str()) {
                machine.firmware_version = fw_version.to_string();
            }
        }
        
        Ok(machine)
    }

    /// Turn on a machine
    pub async fn turn_on_machine(&self, serial_number: &str) -> Result<bool> {
        let url = format!("{}/things/{}/command/CoffeeMachineChangeMode", CUSTOMER_APP_URL, serial_number);
        let data = serde_json::json!({
            "mode": "BrewingMode"
        });
        
        let _response: Value = self.post(&url, Some(data)).await?;
        // Here we should check the command response, but for simplicity we'll assume success
        Ok(true)
    }

    /// Turn off a machine
    pub async fn turn_off_machine(&self, serial_number: &str) -> Result<bool> {
        let url = format!("{}/things/{}/command/CoffeeMachineChangeMode", CUSTOMER_APP_URL, serial_number);
        let data = serde_json::json!({
            "mode": "StandBy"
        });
        
        let _response: Value = self.post(&url, Some(data)).await?;
        // Here we should check the command response, but for simplicity we'll assume success
        Ok(true)
    }
}