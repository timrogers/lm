use clap::{Parser, Subcommand};
use lm::{LaMarzocoClient, Config, Result};

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
    On,
    /// Turn off an espresso machine
    Off,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let config = Config::load().await?;
    let client = LaMarzocoClient::new(config).await?;

    match cli.command {
        Commands::Status => {
            let machines = client.get_machines().await?;
            for machine in machines {
                println!("Machine: {} ({})", machine.model, machine.serial_number);
                println!("  Status: {}", if machine.is_on { "On" } else { "Off" });
                println!("  Ready: {}", if machine.is_ready { "Yes" } else { "No" });
                println!();
            }
        }
        Commands::On => {
            let machines = client.get_machines().await?;
            if let Some(machine) = machines.first() {
                println!("Turning on machine {}...", machine.serial_number);
                client.set_power(&machine.serial_number, true).await?;
                println!("Machine turned on successfully");
            } else {
                println!("No machines found");
            }
        }
        Commands::Off => {
            let machines = client.get_machines().await?;
            if let Some(machine) = machines.first() {
                println!("Turning off machine {}...", machine.serial_number);
                client.set_power(&machine.serial_number, false).await?;
                println!("Machine turned off successfully");
            } else {
                println!("No machines found");
            }
        }
    }

    Ok(())
}
