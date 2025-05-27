use anyhow::Result;
use clap::{Parser, Subcommand};
use lm::LaMarzoccoClient;
use std::io::{self, Write};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Status {
        #[arg(long, help = "Serial number of specific machine")]
        serial: Option<String>,
    },
    On {
        #[arg(help = "Serial number of machine to turn on")]
        serial: String,
    },
    Off {
        #[arg(help = "Serial number of machine to turn off")]
        serial: String,
    },
}

async fn ensure_authentication(client: &mut LaMarzoccoClient) -> Result<()> {
    client.load_credentials()?;

    if !client.is_authenticated() {
        print!("Email: ");
        io::stdout().flush()?;
        let mut email = String::new();
        io::stdin().read_line(&mut email)?;
        let email = email.trim();

        let password = rpassword::prompt_password("Password: ")?;

        client.authenticate(email, &password).await?;
        println!("Authentication successful!");
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut client = LaMarzoccoClient::new();

    ensure_authentication(&mut client).await?;

    match &cli.command {
        Commands::Status { serial } => {
            if let Some(serial_number) = serial {
                let status = client.get_machine_status(serial_number).await?;
                println!("Machine Status: {:?}", status);
                println!("Machine: {} ({})", status.name, status.model);
                println!("Serial: {}", status.serial_number);
                println!("Turned on: {}", if status.turned_on { "Yes" } else { "No" });
                println!("Ready: {}", if status.ready { "Yes" } else { "No" });
            } else {
                let machines = client.get_machines().await?;
                if machines.is_empty() {
                    println!("No machines found");
                } else {
                    println!("Connected machines:");
                    for machine in &machines {
                        let status = client.get_machine_status(&machine.serial_number).await?;
                        println!("Machine Status: {:?}", status);
                        println!(
                            "  {} ({}) - Serial: {}",
                            status.name, status.model, status.serial_number
                        );
                        println!(
                            "    Turned on: {}",
                            if status.turned_on { "Yes" } else { "No" }
                        );
                        println!("    Ready: {}", if status.ready { "Yes" } else { "No" });
                        println!();
                    }
                }
            }
        }
        Commands::On { serial } => {
            client.set_machine_power(serial, true).await?;
            println!("Turning on machine {}", serial);
        }
        Commands::Off { serial } => {
            client.set_machine_power(serial, false).await?;
            println!("Turning off machine {}", serial);
        }
    }

    Ok(())
}
