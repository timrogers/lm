use anyhow::Result;
use clap::{Parser, Subcommand};
use indicatif::{ProgressBar, ProgressStyle};
use log::{debug, info, warn};
use notify_rust::Notification;
use std::time::Duration;
use tabled::{Table, Tabled};

mod auth;
mod client;
mod types;

use client::LaMarzoccoClient;

#[derive(Parser)]
#[command(name = "lm")]
#[command(about = "A CLI for controlling La Marzocco espresso machines")]
#[command(version)]
struct Cli {
    /// Username for La Marzocco account
    #[arg(long, env = "LM_USERNAME")]
    username: Option<String>,

    /// Password for La Marzocco account
    #[arg(long, env = "LM_PASSWORD")]
    password: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Turn on the espresso machine
    On {
        /// Serial number of the machine (optional if only one machine)
        #[arg(long)]
        serial: Option<String>,
        /// Wait for the machine to be ready before returning
        #[arg(long)]
        wait: bool,
    },
    /// Turn off the espresso machine (standby mode)
    Off {
        /// Serial number of the machine (optional if only one machine)
        #[arg(long)]
        serial: Option<String>,
    },
    /// List all machines connected to the account
    Machines,
}

#[derive(Tabled)]
struct MachineRow {
    #[tabled(rename = "Model")]
    model: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Serial")]
    serial: String,
    #[tabled(rename = "Status")]
    status: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let cli = Cli::parse();

    // Validate credentials
    let username = cli.username.ok_or_else(|| {
        anyhow::anyhow!(
            "Username is required. Provide via --username or LM_USERNAME environment variable."
        )
    })?;

    let password = cli.password.ok_or_else(|| {
        anyhow::anyhow!(
            "Password is required. Provide via --password or LM_PASSWORD environment variable."
        )
    })?;

    // Create client and authenticate
    let mut client = LaMarzoccoClient::new();

    info!("Authenticating with La Marzocco...");
    client.authenticate(&username, &password).await?;
    debug!("Authentication successful");

    match cli.command {
        Commands::Machines => {
            info!("Fetching machine list...");
            let machines = client.get_machines().await?;

            if machines.is_empty() {
                println!("No machines found for this account.");
                return Ok(());
            }

            let mut rows: Vec<MachineRow> = Vec::new();

            for machine in &machines {
                let status = machine.get_status_display(&client).await;

                rows.push(MachineRow {
                    model: machine
                        .model
                        .clone()
                        .unwrap_or_else(|| "Unknown".to_string()),
                    name: machine
                        .name
                        .clone()
                        .unwrap_or_else(|| "Unnamed".to_string()),
                    serial: machine.serial_number.clone(),
                    status,
                });
            }

            let table = Table::new(&rows);
            println!("{}", table);
        }
        Commands::On { serial, wait } => {
            let machine_serial = match serial {
                Some(s) => s,
                None => {
                    let machines = client.get_machines().await?;
                    if machines.is_empty() {
                        return Err(anyhow::anyhow!("No machines found for this account."));
                    }
                    if machines.len() > 1 {
                        return Err(anyhow::anyhow!(
                            "Multiple machines found. Please specify --serial."
                        ));
                    }
                    machines[0].serial_number.clone()
                }
            };

            info!("Turning on machine {}", machine_serial);
            client.turn_on_machine(&machine_serial).await?;

            if wait {
                wait_for_machine_ready(&client, &machine_serial).await?;
            } else {
                println!("Machine {} turned on successfully.", machine_serial);
            }
        }
        Commands::Off { serial } => {
            let machine_serial = match serial {
                Some(s) => s,
                None => {
                    let machines = client.get_machines().await?;
                    if machines.is_empty() {
                        return Err(anyhow::anyhow!("No machines found for this account."));
                    }
                    if machines.len() > 1 {
                        return Err(anyhow::anyhow!(
                            "Multiple machines found. Please specify --serial."
                        ));
                    }
                    machines[0].serial_number.clone()
                }
            };

            info!("Turning off machine {}", machine_serial);
            client.turn_off_machine(&machine_serial).await?;
            println!("Machine {} turned off (standby mode).", machine_serial);
        }
    }

    Ok(())
}

/// Wait for a machine to be ready with exponential backoff polling
///
/// This function polls the machine status at increasing intervals:
/// - Starts with 2-second delays
/// - Doubles the delay after each check (exponential backoff)
/// - Caps at 30-second delays
/// - Shows an animated spinner with status updates
/// - Returns when machine shows "On (Ready)" status
/// - Treats "Standby" as normal startup state (not an error)
async fn wait_for_machine_ready(client: &LaMarzoccoClient, machine_serial: &str) -> Result<()> {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    spinner.enable_steady_tick(Duration::from_millis(100));
    spinner.set_message("Waiting for machine to be ready...");

    let mut delay = Duration::from_secs(2); // Start with 2 second delay
    let max_delay = Duration::from_secs(30); // Maximum 30 second delay

    tokio::time::sleep(delay).await;

    loop {
        match client.get_machine_status(machine_serial).await {
            Ok(status) => {
                let status_string = status.get_status_string();

                if status_string == "On (Ready)" {
                    spinner.finish_with_message("Machine is ready! ☕");

                    // Send desktop notification
                    if let Err(e) = Notification::new()
                        .summary("La Marzocco Machine Ready")
                        .body("Your espresso machine is ready to brew! ☕")
                        .icon("coffee")
                        .timeout(5000) // 5 seconds
                        .show()
                    {
                        warn!("Failed to send notification: {}", e);
                    }

                    return Ok(());
                } else if status_string.starts_with("On (Ready in") {
                    spinner.set_message(format!("Machine heating up - {}", status_string));
                } else if status_string == "On (Ready in < 1 min)" {
                    spinner.set_message("Machine almost ready...");
                } else if status_string == "On (Heating)" {
                    spinner.set_message("Machine heating up...");
                } else if status_string == "Standby" {
                    spinner.set_message("Machine starting up...");
                } else {
                    spinner.set_message(format!("Machine status: {}", status_string));
                }
            }
            Err(e) => {
                spinner.set_message(format!("Error checking status: {}", e));
            }
        }

        // Wait with current delay
        tokio::time::sleep(delay).await;

        // Exponential backoff with maximum delay
        if delay < max_delay {
            delay = std::cmp::min(delay * 2, max_delay);
        }
    }
}

#[cfg(test)]
mod wait_tests {
    use std::time::Duration;

    #[test]
    fn test_exponential_backoff_calculation() {
        // Test the exponential backoff logic used in wait_for_machine_ready
        let mut delay = Duration::from_secs(2);
        let max_delay = Duration::from_secs(30);

        // First delay should be 2 seconds
        assert_eq!(delay, Duration::from_secs(2));

        // Second delay should be 4 seconds
        delay = std::cmp::min(delay * 2, max_delay);
        assert_eq!(delay, Duration::from_secs(4));

        // Third delay should be 8 seconds
        delay = std::cmp::min(delay * 2, max_delay);
        assert_eq!(delay, Duration::from_secs(8));

        // Fourth delay should be 16 seconds
        delay = std::cmp::min(delay * 2, max_delay);
        assert_eq!(delay, Duration::from_secs(16));

        // Fifth delay should be 30 seconds (capped at max)
        delay = std::cmp::min(delay * 2, max_delay);
        assert_eq!(delay, Duration::from_secs(30));

        // Sixth delay should remain at 30 seconds (still capped)
        delay = std::cmp::min(delay * 2, max_delay);
        assert_eq!(delay, Duration::from_secs(30));
    }
}
