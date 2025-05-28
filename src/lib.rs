pub mod auth;
pub mod client;
pub mod config;
pub mod error;
pub mod models;

pub use client::LaMarzoccoClient;
pub use error::Error;
pub use models::{Machine, MachineStatus};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        // Basic test to ensure compilation
        assert_eq!(2 + 2, 4);
    }
}
