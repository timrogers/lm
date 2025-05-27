use crate::auth::Auth;
use crate::error::{LaMarzoccoError, Result};
use crate::models::{CommandResponse, Thing, ThingDashboardConfig};
use reqwest::{Client as ReqwestClient, Method};
use serde::de::DeserializeOwned;
use serde_json::json;

const CUSTOMER_APP_URL: &str = "https://cms-api.lamarzocco.io/api/v2/home/lm";

/// Client for interacting with the La Marzocco API
pub struct Client {
    auth: Auth,
    client: ReqwestClient,
}

impl Client {
    /// Create a new client
    pub fn new() -> Self {
        Client {
            auth: Auth::new(),
            client: ReqwestClient::new(),
        }
    }

    /// Authenticate with username and password
    pub async fn authenticate(&mut self, username: &str, password: &str) -> Result<()> {
        self.auth.authenticate(username, password).await
    }

    /// Make an API request
    async fn request<T: DeserializeOwned>(
        &mut self,
        method: Method,
        endpoint: &str,
        body: Option<serde_json::Value>,
    ) -> Result<T> {
        // Get a valid access token
        let access_token = self.auth.get_access_token().await?;
        
        // Build the request
        let url = format!("{}{}", CUSTOMER_APP_URL, endpoint);
        let mut req = self
            .client
            .request(method, &url)
            .header("Authorization", format!("Bearer {}", access_token));

        if let Some(json_body) = body {
            req = req.json(&json_body);
        }

        // Send the request
        let response = req.send().await?;

        // Handle the response
        if response.status().is_success() {
            let data = response.json::<T>().await?;
            Ok(data)
        } else {
            Err(LaMarzoccoError::ApiError {
                status_code: response.status().as_u16(),
                message: response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string()),
            })
        }
    }

    /// List all machines
    pub async fn list_machines(&mut self) -> Result<Vec<Thing>> {
        self.request::<Vec<Thing>>(Method::GET, "/things", None).await
    }

    /// Get dashboard for a machine
    pub async fn get_machine_dashboard(&mut self, serial_number: &str) -> Result<ThingDashboardConfig> {
        let endpoint = format!("/things/{}/dashboard", serial_number);
        self.request::<ThingDashboardConfig>(Method::GET, &endpoint, None).await
    }

    /// Set power state of a machine
    pub async fn set_power(&mut self, serial_number: &str, enabled: bool) -> Result<bool> {
        let command = "CoffeeMachineChangeMode";
        let mode = if enabled { "BrewingMode" } else { "StandBy" };
        
        let endpoint = format!("/things/{}/command/{}", serial_number, command);
        let body = json!({ "mode": mode });
        
        let response: Vec<CommandResponse> = self
            .request(Method::POST, &endpoint, Some(body))
            .await?;
            
        // Consider command successful if we get a response
        Ok(!response.is_empty())
    }
}