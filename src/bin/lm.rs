use anyhow::Result;
use clap::{Parser, Subcommand};
use lm::{create_initial_config, LaMarzoccoClient};
use std::process;
use tokio::time::Duration;

#[derive(Parser)]
#[clap(author, version, about = "Control La Marzocco Home espresso machines")]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Login to La Marzocco Home
    Login {
        /// Email address
        #[clap(long)]
        email: String,

        /// Password
        #[clap(long)]
        password: String,
    },

    /// View machine status
    Status,

    /// Turn machine on
    On {
        /// Machine serial number (if multiple machines)
        #[clap(long)]
        serial: Option<String>,
    },

    /// Turn machine off
    Off {
        /// Machine serial number (if multiple machines)
        #[clap(long)]
        serial: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Login { email, password } => {
            println!("Logging in to La Marzocco Home as {}...", email);

            // Create initial configuration
            if let Err(err) = create_initial_config(email.to_string(), password.to_string()) {
                eprintln!("Error creating configuration: {}", err);
                process::exit(1);
            }

            // Test authentication
            match LaMarzoccoClient::new(email.to_string(), password.to_string()).await {
                Ok(mut client) => match client.list_machines().await {
                    Ok(_) => {
                        println!("Login successful!");
                    }
                    Err(err) => {
                        eprintln!("Authentication error: {}", err);
                        process::exit(1);
                    }
                },
                Err(err) => {
                    eprintln!("Error creating client: {}", err);
                    process::exit(1);
                }
            }
        }
        Commands::Status => {
            println!("Fetching machine status...");

            match LaMarzoccoClient::from_config().await {
                Ok(mut client) => match client.list_machines().await {
                    Ok(machines) => {
                        if machines.is_empty() {
                            println!("No machines found");
                        } else {
                            for machine in machines {
                                println!("Machine: {} ({})", machine.name, machine.model_name);
                                println!("Serial Number: {}", machine.serial_number);

                                let status_text = match machine.status {
                                    lm::MachineStatus::PoweredOn => "Turned on, ready",
                                    lm::MachineStatus::Standby => "Standby/Off",
                                    lm::MachineStatus::Brewing => "Brewing",
                                    lm::MachineStatus::Off => "Off",
                                };

                                println!("Status: {}", status_text);
                                println!();
                            }
                        }
                    }
                    Err(err) => {
                        eprintln!("Error fetching machines: {}", err);
                        process::exit(1);
                    }
                },
                Err(err) => {
                    eprintln!("Error loading configuration: {}", err);
                    process::exit(1);
                }
            }
        }
        Commands::On { serial } => {
            println!("Turning machine on...");

            match LaMarzoccoClient::from_config().await {
                Ok(mut client) => {
                    let machine_serial = match serial {
                        Some(s) => s.clone(),
                        None => {
                            // If no serial provided, use the first machine
                            match client.list_machines().await {
                                Ok(machines) => {
                                    if machines.is_empty() {
                                        eprintln!("No machines found");
                                        process::exit(1);
                                    }
                                    machines[0].serial_number.clone()
                                }
                                Err(err) => {
                                    eprintln!("Error fetching machines: {}", err);
                                    process::exit(1);
                                }
                            }
                        }
                    };

                    match client.turn_on(&machine_serial).await {
                        Ok(_) => {
                            println!("Machine turning on...");

                            // Wait briefly to check status
                            tokio::time::sleep(Duration::from_secs(2)).await;

                            match client.list_machines().await {
                                Ok(machines) => {
                                    for machine in machines {
                                        if machine.serial_number == machine_serial {
                                            let status_text = match machine.status {
                                                lm::MachineStatus::PoweredOn => "Turned on, ready",
                                                lm::MachineStatus::Brewing => "Brewing",
                                                _ => "Turning on...",
                                            };

                                            println!("Current status: {}", status_text);
                                            break;
                                        }
                                    }
                                }
                                Err(err) => {
                                    eprintln!("Error checking machine status: {}", err);
                                }
                            }
                        }
                        Err(err) => {
                            eprintln!("Error turning machine on: {}", err);
                            process::exit(1);
                        }
                    }
                }
                Err(err) => {
                    eprintln!("Error loading configuration: {}", err);
                    process::exit(1);
                }
            }
        }
        Commands::Off { serial } => {
            println!("Turning machine off...");

            match LaMarzoccoClient::from_config().await {
                Ok(mut client) => {
                    let machine_serial = match serial {
                        Some(s) => s.clone(),
                        None => {
                            // If no serial provided, use the first machine
                            match client.list_machines().await {
                                Ok(machines) => {
                                    if machines.is_empty() {
                                        eprintln!("No machines found");
                                        process::exit(1);
                                    }
                                    machines[0].serial_number.clone()
                                }
                                Err(err) => {
                                    eprintln!("Error fetching machines: {}", err);
                                    process::exit(1);
                                }
                            }
                        }
                    };

                    match client.turn_off(&machine_serial).await {
                        Ok(_) => {
                            println!("Machine turning off...");

                            // Wait briefly to check status
                            tokio::time::sleep(Duration::from_secs(2)).await;

                            match client.list_machines().await {
                                Ok(machines) => {
                                    for machine in machines {
                                        if machine.serial_number == machine_serial {
                                            let status_text = match machine.status {
                                                lm::MachineStatus::Standby => "Standby/Off",
                                                lm::MachineStatus::Off => "Off",
                                                _ => "Turning off...",
                                            };

                                            println!("Current status: {}", status_text);
                                            break;
                                        }
                                    }
                                }
                                Err(err) => {
                                    eprintln!("Error checking machine status: {}", err);
                                }
                            }
                        }
                        Err(err) => {
                            eprintln!("Error turning machine off: {}", err);
                            process::exit(1);
                        }
                    }
                }
                Err(err) => {
                    eprintln!("Error loading configuration: {}", err);
                    process::exit(1);
                }
            }
        }
    }

    Ok(())
}
