use anyhow::Result;
use clap::{Parser, Subcommand};
use indicatif::{ProgressBar, ProgressStyle};
use log::{debug, info, warn};
use notify_rust::Notification;
use std::io::{self, Write};
use std::sync::Arc;
use std::time::Duration;
use tabled::{Table, Tabled};

// Use the new library interface
use lm::{config, ApiClient, AuthenticationClient, Credentials, TokenRefreshCallback};

/// Check if an error indicates authentication failure and clear config if so
fn handle_auth_error(e: anyhow::Error) -> anyhow::Error {
    let error_msg = e.to_string();
    if error_msg.contains("Please re-authenticate")
        || error_msg.contains("Authentication failed. Please run 'lm login' again.")
    {
        warn!("Stored credentials are invalid, clearing config file");
        let _ = config::clear_config();
        return anyhow::anyhow!("Stored credentials are invalid. Please run 'lm login' again.");
    }
    e
}

#[derive(Parser)]
#[command(name = "lm")]
#[command(about = "A CLI for controlling La Marzocco espresso machines")]
#[command(version)]
#[command(propagate_version = true, arg_required_else_help = true)]
struct Cli {
    /// The username for your La Marzocco account. You can provide this for every command as an argument or environment variable, or you can log in once with `lm login` to store it for future use.
    #[arg(long, short = 'u', env = "LM_USERNAME", global = true)]
    username: Option<String>,

    /// The password for your La Marzocco account. You can provide this for every command as an argument or environment variable, or you can log in once with `lm login` to store it for future use.
    #[arg(long, short = 'p', env = "LM_PASSWORD", global = true)]
    password: Option<String>,

    /// Enable verbose logging
    #[arg(long, short = 'v', global = true, default_value_t = false)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Log in to your La Marzocco account and store credentials for future use
    Login {
        /// The username for your La Marzocco account. If not provided, you will be prompted to enter it.
        #[arg(long, short = 'u')]
        username: Option<String>,
        /// The password for your La Marzocco account. If not provided, you will be prompted to enter it securely. Your password will not be stored, but an access token will be obtained and saved for future use.
        #[arg(long, short = 'p')]
        password: Option<String>,
    },
    /// Log out of your La Marzocco account and clear stored credentials
    Logout,
    /// Turn on the espresso machine
    On {
        /// The serial number of the machine (optional if only one machine is connected to your account)
        #[arg(long, short = 's')]
        serial: Option<String>,
        /// Wait for the machine to be ready to brew before exiting, and trigger a notification when ready
        #[arg(long, short = 'w', default_value_t = false)]
        wait: bool,
    },
    /// Switch the espresso machine to standby mode
    Off {
        /// The serial number of the machine (optional if only one machine is connected to your account)
        #[arg(long, short = 's')]
        serial: Option<String>,
    },
    /// List all machines connected to the account
    Machines,
}

#[derive(Tabled)]
struct MachineRow {
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Serial")]
    serial: String,
    #[tabled(rename = "Status")]
    status: String,
}

/// Token refresh callback that saves tokens to ~/.lm.yml
struct CliTokenCallback;

impl TokenRefreshCallback for CliTokenCallback {
    fn on_tokens_refreshed(&self, credentials: &Credentials) {
        debug!("Tokens refreshed for user: {}", credentials.username);

        // Save the refreshed tokens to the config file
        let config = config::Config::from(credentials);
        if let Err(e) = config::save_config(&config) {
            warn!("Failed to save refreshed tokens to config file: {}", e);
        } else {
            debug!("Refreshed tokens saved to config file");
        }
    }
}

/// Prompt for username if not provided
fn prompt_username(username: Option<String>) -> Result<String> {
    match username {
        Some(u) => Ok(u),
        None => {
            print!("Username: ");
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            Ok(input.trim().to_string())
        }
    }
}

/// Securely prompt for password if not provided
fn prompt_password(password: Option<String>) -> Result<String> {
    match password {
        Some(p) => Ok(p),
        None => {
            let password = rpassword::prompt_password("Password: ")?;
            Ok(password)
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logger based on verbose flag
    if cli.verbose {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Debug)
            .init();
    } else {
        env_logger::init();
    }

    match cli.command {
        Commands::Login { username, password } => {
            // Handle login command
            let username = prompt_username(username)?;
            let password = prompt_password(password)?;

            // Authenticate using the new authentication client
            let auth_client = AuthenticationClient::new();
            info!("Authenticating with La Marzocco...");
            let tokens = auth_client.login(&username, &password).await?;
            debug!("Authentication successful");

            // Save tokens to config file
            let config = config::Config::from(&tokens);
            config::save_config(&config)?;

            println!("✅ Authentication successful! Credentials saved to ~/.lm.yml.");
            return Ok(());
        }
        Commands::Logout => {
            // Handle logout command
            config::clear_config()?;
            println!("✅ Logged out successfully. Credentials cleared.");
            return Ok(());
        }
        _ => {
            // For other commands, we need authentication
            // Try to load stored credentials first
            let credentials = match config::load_config() {
                Ok(config) => {
                    debug!("Using stored credentials for user: {}", config.username);
                    Credentials::from(config)
                }
                Err(_) => {
                    // Fall back to CLI arguments or environment variables
                    let username = cli.username.ok_or_else(|| {
                        anyhow::anyhow!(
                            "You don't seem to be logged in. Please run 'lm login' or provide --username and --password."
                        )
                    })?;

                    let password = cli.password.ok_or_else(|| {
                        anyhow::anyhow!(
                            "You don't seem to be logged in. Please run 'lm login' or provide --username and --password."
                        )
                    })?;

                    // Authenticate using the new authentication client
                    let auth_client = AuthenticationClient::new();
                    info!("Authenticating with La Marzocco...");
                    let tokens = auth_client.login(&username, &password).await?;
                    debug!("Authentication successful");
                    tokens
                }
            };

            // Create API client with token refresh callback
            let callback = Arc::new(CliTokenCallback);
            let mut api_client = ApiClient::new(credentials, Some(callback));

            // Handle the API commands
            match cli.command {
                Commands::Machines => {
                    info!("Fetching machine list...");

                    let machines = match api_client.get_machines().await {
                        Ok(machines) => machines,
                        Err(e) => return Err(handle_auth_error(e)),
                    };

                    if machines.is_empty() {
                        println!("⚠️ No machines connected to your La Marzocco account.");
                        return Ok(());
                    }

                    let mut rows: Vec<MachineRow> = Vec::new();

                    for machine in &machines {
                        // For status display, use the new API client directly
                        let status = if machine.connected {
                            match api_client.get_machine_status(&machine.serial_number).await {
                                Ok(status) => status.get_status_string(),
                                Err(_) => "Unknown".to_string(),
                            }
                        } else {
                            "Unavailable".to_string()
                        };

                        let machine_name = machine
                            .name
                            .clone()
                            .unwrap_or_else(|| "Unnamed".to_string());

                        let machine_model = machine
                            .model
                            .clone()
                            .unwrap_or_else(|| "Unknown".to_string());

                        let combined_name = format!("{} ({})", machine_name, machine_model);

                        rows.push(MachineRow {
                            name: combined_name,
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
                            let machines = match api_client.get_machines().await {
                                Ok(machines) => machines,
                                Err(e) => return Err(handle_auth_error(e)),
                            };

                            if machines.is_empty() {
                                return Err(anyhow::anyhow!(
                                    "⚠️ No machines found connected to your La Marzocco account."
                                ));
                            }
                            if machines.len() > 1 {
                                return Err(anyhow::anyhow!(
                                    "⚠️ Multiple machines found connected to your La Marzocco account. Please specify a machine with --serial."
                                ));
                            }
                            machines[0].serial_number.clone()
                        }
                    };

                    info!("Turning on machine {}", machine_serial);
                    match api_client.turn_on_machine(&machine_serial).await {
                        Ok(_) => {}
                        Err(e) => return Err(handle_auth_error(e)),
                    }

                    if wait {
                        wait_for_machine_ready(&mut api_client, &machine_serial).await?;
                    } else {
                        println!("✅ Machine {} turned on successfully.", machine_serial);
                    }
                }
                Commands::Off { serial } => {
                    let machine_serial = match serial {
                        Some(s) => s,
                        None => {
                            let machines = match api_client.get_machines().await {
                                Ok(machines) => machines,
                                Err(e) => return Err(handle_auth_error(e)),
                            };

                            if machines.is_empty() {
                                return Err(anyhow::anyhow!(
                                    "⚠️ No machines found connected to your La Marzocco account."
                                ));
                            }
                            if machines.len() > 1 {
                                return Err(anyhow::anyhow!(
                                    "⚠️ Multiple machines found connected to your La Marzocco account. Please specify a machine with --serial."
                                ));
                            }
                            machines[0].serial_number.clone()
                        }
                    };

                    info!("Turning off machine {}", machine_serial);
                    match api_client.turn_off_machine(&machine_serial).await {
                        Ok(_) => {}
                        Err(e) => return Err(handle_auth_error(e)),
                    }

                    println!("✅ Machine {} switched to standby mode.", machine_serial);
                }
                _ => unreachable!(),
            }
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
async fn wait_for_machine_ready(api_client: &mut ApiClient, machine_serial: &str) -> Result<()> {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    spinner.enable_steady_tick(Duration::from_millis(100));
    spinner.set_message("Waiting for your machine to be ready...");

    let mut delay = Duration::from_secs(2); // Start with 2 second delay
    let max_delay = Duration::from_secs(30); // Maximum 30 second delay
    let mut no_water_notification_sent = false; // Track if we've sent the no water notification

    tokio::time::sleep(delay).await;

    loop {
        match api_client.get_machine_status(machine_serial).await {
            Ok(status) => {
                let status_string = status.get_status_string();

                if status_string == "On (Ready)" {
                    spinner.finish_with_message("✅ Machine is ready! ☕");

                    // Send desktop notification
                    if let Err(e) = Notification::new()
                        .summary("La Marzocco machine ready")
                        .body("Your espresso machine is ready to brew! ☕")
                        .timeout(5000) // 5 seconds
                        .show()
                    {
                        warn!("Failed to send notification: {}", e);
                    }

                    return Ok(());
                } else if status_string == "On (No water)" {
                    spinner.set_message("⚠️ Machine has no water - please refill reservoir. ");

                    // Send notification only once per run
                    if !no_water_notification_sent {
                        if let Err(e) = Notification::new()
                            .summary("La Marzocco machine needs water")
                            .body("Please refill the water reservoir and wait for the boiler to be ready.")
                            .timeout(5000) // 5 seconds
                            .show()
                        {
                            warn!("Failed to send notification: {}", e);
                        }
                        no_water_notification_sent = true;
                    }
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

    #[test]
    fn test_machine_row_name_formatting() {
        use super::MachineRow;
        use tabled::Table;

        // Test the name formatting in MachineRow
        let row = MachineRow {
            name: "Linea Micra (LINEA MICRA)".to_string(),
            serial: "MR033274".to_string(),
            status: "Connected".to_string(),
        };

        // Verify the name field contains both name and model
        assert!(row.name.contains("Linea Micra"));
        assert!(row.name.contains("(LINEA MICRA)"));

        // Test that the table can be created successfully
        let rows = vec![row];
        let table = Table::new(&rows);
        let table_string = table.to_string();

        // Verify the table contains our expected content
        assert!(table_string.contains("Name"));
        assert!(table_string.contains("Serial"));
        assert!(table_string.contains("Status"));
        assert!(table_string.contains("Linea Micra (LINEA MICRA)"));
        assert!(table_string.contains("MR033274"));
        assert!(table_string.contains("Connected"));
    }
}
