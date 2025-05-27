use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use lm::{Client, Machine};
use std::process::exit;

#[derive(Parser)]
#[command(author, version, about = "Control La Marzocco espresso machines")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Login to La Marzocco cloud
    Login {
        /// Email address
        #[arg(long)]
        email: String,
        /// Password
        #[arg(long)]
        password: String,
    },
    /// Show status of connected espresso machines
    Status,
    /// Turn on the espresso machine
    On {
        /// Serial number of the machine (if you have multiple machines)
        #[arg(long)]
        serial: Option<String>,
    },
    /// Turn off the espresso machine
    Off {
        /// Serial number of the machine (if you have multiple machines)
        #[arg(long)]
        serial: Option<String>,
    },
}

/// Configure a CLI client
async fn configure_client() -> Result<Client> {
    let client = Client::new();
    
    // We don't need to do anything else here - token loading is handled automatically
    // when needed by the client
    
    Ok(client)
}

/// Get the machine to operate on (handling the case of multiple machines)
async fn get_machine(client: &mut Client, serial: Option<String>) -> Result<Machine> {
    // Get all machines
    let machines = client.list_machines().await.map_err(|e| anyhow!(e.to_string()))?;
    
    if machines.is_empty() {
        return Err(anyhow!("No machines found"));
    }
    
    // If serial is provided, find that specific machine
    if let Some(serial_number) = serial {
        let found = machines.iter().find(|m| m.serial_number == serial_number);
        
        if let Some(machine) = found {
            return Machine::get_status(client, &machine.serial_number)
                .await
                .map_err(|e| anyhow!(e.to_string()));
        } else {
            return Err(anyhow!("Machine with serial number {} not found", serial_number));
        }
    }
    
    // If only one machine, use that
    if machines.len() == 1 {
        return Machine::get_status(client, &machines[0].serial_number)
            .await
            .map_err(|e| anyhow!(e.to_string()));
    }
    
    // Multiple machines, but no serial specified
    Err(anyhow!(
        "Multiple machines found. Please specify a serial number with --serial"
    ))
}

/// Display status of a machine
fn display_status(machine: &Machine) {
    println!("Serial number: {}", machine.serial_number);
    println!("Model: {}", machine.model_name);
    println!("Power status: {}", if machine.turned_on { "ON" } else { "OFF" });
    println!("Ready status: {}", if machine.is_ready { "READY" } else { "NOT READY" });
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Login { email, password }) => {
            let mut client = Client::new();
            match client.authenticate(&email, &password).await {
                Ok(_) => {
                    println!("Successfully logged in and saved credentials");
                }
                Err(e) => {
                    eprintln!("Login failed: {}", e);
                    exit(1);
                }
            }
        }
        Some(Commands::Status) => {
            let mut client = configure_client().await?;
            let machines = client.list_machines().await.map_err(|e| anyhow!(e.to_string()))?;
            
            if machines.is_empty() {
                println!("No machines found");
                return Ok(());
            }
            
            for machine_info in &machines {
                let machine = Machine::get_status(&mut client, &machine_info.serial_number)
                    .await
                    .map_err(|e| anyhow!(e.to_string()))?;
                
                display_status(&machine);
                
                // Add separator between machines
                if machines.len() > 1 && machine_info.serial_number != machines.last().unwrap().serial_number {
                    println!("---");
                }
            }
        }
        Some(Commands::On { serial }) => {
            let mut client = configure_client().await?;
            let machine = get_machine(&mut client, serial).await?;
            
            println!("Turning on machine {}...", machine.serial_number);
            match machine.turn_on(&mut client).await {
                Ok(_) => println!("Machine turned on successfully"),
                Err(e) => {
                    eprintln!("Failed to turn on machine: {}", e);
                    exit(1);
                }
            }
        }
        Some(Commands::Off { serial }) => {
            let mut client = configure_client().await?;
            let machine = get_machine(&mut client, serial).await?;
            
            println!("Turning off machine {}...", machine.serial_number);
            match machine.turn_off(&mut client).await {
                Ok(_) => println!("Machine turned off successfully"),
                Err(e) => {
                    eprintln!("Failed to turn off machine: {}", e);
                    exit(1);
                }
            }
        }
        None => {
            // No command provided, show help
            Cli::parse_from(&["lm", "--help"]);
        }
    }

    Ok(())
}
