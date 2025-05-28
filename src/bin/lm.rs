use anyhow::Result;
use clap::{Parser, Subcommand};
use lm::{config, LaMarzoccoClient};
use std::io::{self, Write};

#[derive(Parser)]
#[command(name = "lm")]
#[command(about = "A CLI for controlling La Marzocco Home espresso machines")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// View the status of connected espresso machines
    Status,
    /// Turn on an espresso machine
    On {
        /// Serial number of the machine (optional, uses first machine if omitted)
        #[arg(short, long)]
        serial: Option<String>,
    },
    /// Turn off an espresso machine
    Off {
        /// Serial number of the machine (optional, uses first machine if omitted)
        #[arg(short, long)]
        serial: Option<String>,
    },
    /// Login with email and password
    Login,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Login => handle_login().await,
        Commands::Status => handle_status().await,
        Commands::On { serial } => handle_power(true, serial).await,
        Commands::Off { serial } => handle_power(false, serial).await,
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    Ok(())
}

async fn handle_login() -> Result<()> {
    print!("Email: ");
    io::stdout().flush()?;
    let mut email = String::new();
    io::stdin().read_line(&mut email)?;
    let email = email.trim();

    print!("Password: ");
    io::stdout().flush()?;
    let mut password = String::new();
    io::stdin().read_line(&mut password)?;
    let password = password.trim();

    let mut client = LaMarzoccoClient::new();
    
    println!("Authenticating...");
    client.authenticate(email, password).await?;
    
    config::save_config(client.get_config())?;
    println!("Login successful! Credentials saved to ~/.lm.yml");
    
    Ok(())
}

async fn handle_status() -> Result<()> {
    let config = config::load_config()?;
    let mut client = LaMarzoccoClient::with_config(config);
    
    println!("Fetching machine status...");
    let machines = client.get_machines().await?;
    
    if machines.is_empty() {
        println!("No machines found.");
        return Ok(());
    }
    
    println!("\nConnected Machines:");
    println!("==================");
    for machine in machines {
        println!("Name: {}", machine.name);
        println!("Model: {}", machine.model);
        println!("Serial: {}", machine.serial_number);
        println!("Power: {}", if machine.is_on { "ON" } else { "OFF" });
        println!("Ready: {}", if machine.is_ready { "YES" } else { "NO" });
        println!("---");
    }
    
    // Save updated config (in case tokens were refreshed)
    config::save_config(client.get_config())?;
    
    Ok(())
}

async fn handle_power(enabled: bool, serial: Option<String>) -> Result<()> {
    let config = config::load_config()?;
    let mut client = LaMarzoccoClient::with_config(config);
    
    let machines = client.get_machines().await?;
    
    if machines.is_empty() {
        println!("No machines found.");
        return Ok(());
    }
    
    let target_machine = if let Some(serial) = serial {
        machines.iter()
            .find(|m| m.serial_number == serial)
            .ok_or_else(|| anyhow::anyhow!("Machine with serial '{}' not found", serial))?
    } else {
        &machines[0]
    };
    
    let action = if enabled { "on" } else { "off" };
    println!("Turning {} machine '{}'...", action, target_machine.name);
    
    client.set_machine_power(&target_machine.serial_number, enabled).await?;
    
    println!("Machine '{}' turned {}!", target_machine.name, action);
    
    // Save updated config (in case tokens were refreshed)
    config::save_config(client.get_config())?;
    
    Ok(())
}
