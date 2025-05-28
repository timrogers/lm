# lm

A CLI and Rust library for controlling La Marzocco Home espresso machines

## Installation

```bash
cargo install --path .
```

## Configuration

Create a `.lm.yml` file in your home directory with your La Marzocco account credentials:

```yaml
email: your-email@example.com
password: your-password
```

The CLI will automatically handle authentication tokens and store them in the same file.

## Usage

### View machine status
```bash
lm status
```

Shows the status of all connected espresso machines, including:
- Model name
- Serial number  
- Power status (On/Off)
- Ready status (Yes/No)

### Turn machine on
```bash
lm on
```

Turns on the first available espresso machine.

### Turn machine off  
```bash
lm off
```

Turns off the first available espresso machine.

## Library Usage

```rust
use lm::{LaMarzocoClient, Config};

#[tokio::main]
async fn main() -> lm::Result<()> {
    let config = Config::load().await?;
    let client = LaMarzocoClient::new(config).await?;
    
    // Get machine status
    let machines = client.get_machines().await?;
    for machine in machines {
        println!("Machine: {} ({})", machine.model, machine.serial_number);
        println!("  Status: {}", if machine.is_on { "On" } else { "Off" });
        println!("  Ready: {}", if machine.is_ready { "Yes" } else { "No" });
    }
    
    // Control machine power
    if let Some(machine) = machines.first() {
        client.set_power(&machine.serial_number, true).await?;  // Turn on
        client.set_power(&machine.serial_number, false).await?; // Turn off
    }
    
    Ok(())
}
```