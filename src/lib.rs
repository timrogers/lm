pub mod auth;
pub mod client;
pub mod error;
pub mod machine;
pub mod models;

// Re-export the main components for easy access
pub use auth::Auth;
pub use client::Client;
pub use error::{LaMarzoccoError, Result};
pub use machine::Machine;

/// Version of the library
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
