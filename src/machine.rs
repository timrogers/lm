use crate::client::Client;
use crate::error::Result;
use crate::models::ThingDashboardConfig;
use serde::Serialize;

/// Represents a La Marzocco espresso machine
#[derive(Debug, Serialize, Clone)]
pub struct Machine {
    /// Serial number
    pub serial_number: String,
    /// Model name
    pub model_name: String,
    /// Whether the machine is turned on
    pub turned_on: bool,
    /// Whether the machine is ready
    pub is_ready: bool,
}

impl Machine {
    /// Create a machine instance from its dashboard configuration
    pub fn from_dashboard(dashboard: ThingDashboardConfig) -> Self {
        // Extract power state from widgets or default to false
        let turned_on = dashboard
            .widgets
            .get("power")
            .and_then(|v| v.get("powerStatus"))
            .and_then(|v| v.as_str())
            .map(|s| s == "1")
            .unwrap_or(false);

        // Extract ready state from widgets or default to false
        let is_ready = dashboard
            .widgets
            .get("power")
            .and_then(|v| v.get("readyState"))
            .and_then(|v| v.as_str())
            .map(|s| s == "1")
            .unwrap_or(false);

        Machine {
            serial_number: dashboard.serial_number,
            model_name: dashboard.model_name.unwrap_or_else(|| "Unknown".to_string()),
            turned_on,
            is_ready,
        }
    }

    /// Get the status of a machine by its serial number
    pub async fn get_status(client: &mut Client, serial_number: &str) -> Result<Self> {
        let dashboard = client.get_machine_dashboard(serial_number).await?;
        Ok(Self::from_dashboard(dashboard))
    }

    /// Turn on the machine
    pub async fn turn_on(&self, client: &mut Client) -> Result<bool> {
        client.set_power(&self.serial_number, true).await
    }

    /// Turn off the machine
    pub async fn turn_off(&self, client: &mut Client) -> Result<bool> {
        client.set_power(&self.serial_number, false).await
    }
}