//! La Marzocco CLI
//!
//! A command-line interface for controlling La Marzocco espresso machines.
//!
//! This library provides functionality to:
//! - Authenticate with La Marzocco cloud service
//! - List machines connected to an account  
//! - Turn machines on and off remotely
//!
//! ## Usage
//!
//! The main functionality is provided through the CLI binary, but the modules
//! can also be used as a library for building other applications.

pub mod auth;
pub mod client;
pub mod types;

pub use client::LaMarzoccoClient;
pub use types::{Machine, MachineCommand, MachineStatus};
