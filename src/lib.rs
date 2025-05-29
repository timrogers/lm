//! La Marzocco CLI and Library
//!
//! A command-line interface and library for controlling La Marzocco espresso machines.
//!
//! This library provides functionality to:
//! - Authenticate with La Marzocco cloud service
//! - List machines connected to an account  
//! - Turn machines on and off remotely
//! - Automatic JWT token management with expiration checking
//! - Token refresh callbacks for custom token persistence
//!
//! ## Library Usage
//!
//! ```rust,no_run
//! use lm::{AuthenticationClient, ApiClient, TokenRefreshCallback, Credentials};
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Authenticate and get tokens
//! let auth_client = AuthenticationClient::new();
//! let tokens = auth_client.login("username", "password").await?;
//!
//! // Create API client with token refresh callback
//! struct MyTokenStorage;
//! impl TokenRefreshCallback for MyTokenStorage {
//!     fn on_tokens_refreshed(&self, credentials: &Credentials) {
//!         // Save refreshed tokens to your storage
//!         println!("Tokens refreshed for user: {}", credentials.username);
//!     }
//! }
//!
//! let callback = Arc::new(MyTokenStorage);
//! let mut api_client = ApiClient::new(tokens, Some(callback));
//!
//! // Use API client for machine operations
//! let machines = api_client.get_machines().await?;
//! if let Some(machine) = machines.first() {
//!     api_client.turn_on_machine(&machine.serial_number).await?;
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## CLI Usage
//!
//! The main functionality is also provided through the CLI binary for direct command-line usage.

pub mod auth;
pub mod client;
pub mod config;
pub mod types;

// Export new library interface
pub use auth::{is_token_expired, ApiClient, AuthenticationClient, TokenRefreshCallback};
pub use types::Credentials;

// Export legacy interface for backward compatibility
pub use client::LaMarzoccoClient;
pub use types::{Machine, MachineCommand, MachineStatus};
