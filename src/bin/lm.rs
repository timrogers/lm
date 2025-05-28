use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use lm::{config, LaMarzoccoClient, Machine, MachineStatus};
use std::process;

#[derive(Parser)]
#[command(name = "lm")]
#[command(author = "Tim Rogers")]
#[command(version = "0.1.0")]
#[command(about = "Control your La Marzocco espresso machine", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// View the status of your espresso machine
    Status {
        /// Serial number of the machine (if you have multiple machines)
        #[arg(short, long)]
        serial_number: Option<String>,
    },

    /// Turn on your espresso machine
    On {
        /// Serial number of the machine (if you have multiple machines)
        #[arg(short, long)]
        serial_number: Option<String>,
    },

    /// Turn off your espresso machine
    Off {
        /// Serial number of the machine (if you have multiple machines)
        #[arg(short, long)]
        serial_number: Option<String>,
    },

    /// Set up the configuration with your credentials
    Setup {
        /// Your La Marzocco Home email address
        #[arg(short, long)]
        email: String,

        /// Your La Marzocco Home password
        #[arg(short, long)]
        password: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Setup { email, password } => {
            setup_config(&email, &password)?;
        }
        Commands::Status { serial_number } => {
            ensure_config()?;
            let machine = get_machine(serial_number).await?;
            print_machine_status(&machine);
        }
        Commands::On { serial_number } => {
            ensure_config()?;
            turn_on_machine(serial_number).await?;
        }
        Commands::Off { serial_number } => {
            ensure_config()?;
            turn_off_machine(serial_number).await?;
        }
    }

    Ok(())
}

/// Set up configuration with credentials
fn setup_config(email: &str, password: &str) -> Result<()> {
    config::create_config(email, password)
        .context("Failed to create configuration")?;
    
    println!("Configuration saved successfully.");
    Ok(())
}

/// Ensure configuration exists
fn ensure_config() -> Result<()> {
    if !config::config_exists() {
        eprintln!("No configuration found. Please run 'lm setup --email <email> --password <password>' first.");
        process::exit(1);
    }
    
    Ok(())
}

/// Get a machine by serial number or the first available
async fn get_machine(serial_number: Option<String>) -> Result<Machine> {
    let client = LaMarzoccoClient::new();
    
    if let Some(serial) = serial_number {
        client.get_machine_status(&serial)
            .await
            .context("Failed to get machine status")
    } else {
        // Get the first machine
        let machines = client.list_machines()
            .await
            .context("Failed to list machines")?;
        
        if machines.is_empty() {
            anyhow::bail!("No machines found");
        }
        
        let first_machine = &machines[0];
        client.get_machine_status(&first_machine.serial_number)
            .await
            .context("Failed to get machine status")
    }
}

/// Print machine status
fn print_machine_status(machine: &Machine) {
    println!("Machine: {} ({})", machine.name, machine.model_name);
    println!("Serial Number: {}", machine.serial_number);
    println!("Firmware Version: {}", machine.firmware_version);
    
    let status_str = match machine.status {
        MachineStatus::PoweredOn => "Powered On",
        MachineStatus::StandBy => "Stand By",
        MachineStatus::Brewing => "Brewing",
        MachineStatus::Off => "Off",
    };
    
    println!("Status: {}", status_str);
}

/// Turn on a machine
async fn turn_on_machine(serial_number: Option<String>) -> Result<()> {
    let machine = get_machine(serial_number.clone()).await?;
    
    // Check if machine is already on
    if machine.status == MachineStatus::PoweredOn || machine.status == MachineStatus::Brewing {
        println!("Machine '{}' is already on.", machine.name);
        return Ok(());
    }
    
    println!("Turning on '{}'...", machine.name);
    
    let client = LaMarzoccoClient::new();
    let result = client.turn_on_machine(&machine.serial_number)
        .await
        .context("Failed to turn on machine")?;
    
    if result {
        println!("Machine turned on successfully.");
    } else {
        println!("Failed to turn on machine.");
    }
    
    Ok(())
}

/// Turn off a machine
async fn turn_off_machine(serial_number: Option<String>) -> Result<()> {
    let machine = get_machine(serial_number.clone()).await?;
    
    // Check if machine is already off
    if machine.status == MachineStatus::StandBy || machine.status == MachineStatus::Off {
        println!("Machine '{}' is already off.", machine.name);
        return Ok(());
    }
    
    println!("Turning off '{}'...", machine.name);
    
    let client = LaMarzoccoClient::new();
    let result = client.turn_off_machine(&machine.serial_number)
        .await
        .context("Failed to turn off machine")?;
    
    if result {
        println!("Machine turned off successfully.");
    } else {
        println!("Failed to turn off machine.");
    }
    
    Ok(())
}
