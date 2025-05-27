# lm

A CLI and Rust library for controlling La Marzocco Home espresso machines.

## Installation

```bash
cargo install --path .
```

## Usage

### Authentication

Before using the CLI, you need to authenticate with your La Marzocco account:

```bash
lm login --email your.email@example.com --password yourpassword
```

This will save your credentials securely in `~/.lm.yml`.

### Commands

#### Check the status of your machine(s)

```bash
lm status
```

This will display information about your machine(s), including power status, model, and serial number.

#### Turn on your machine

```bash
lm on
```

If you have multiple machines:

```bash
lm on --serial YOUR_SERIAL_NUMBER
```

#### Turn off your machine

```bash
lm off
```

If you have multiple machines:

```bash
lm off --serial YOUR_SERIAL_NUMBER
```

## Library

You can also use the `lm` crate as a library in your Rust projects:

```rust
use lm::{Client, Machine};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client
    let mut client = Client::new();

    // Authenticate (only needed once, credentials are saved)
    client.authenticate("your.email@example.com", "yourpassword").await?;

    // List all machines
    let machines = client.list_machines().await?;
    
    // Get status of first machine
    if let Some(machine_info) = machines.first() {
        let machine = Machine::get_status(&mut client, &machine_info.serial_number).await?;
        
        // Turn on the machine
        machine.turn_on(&mut client).await?;
        
        // Turn off the machine
        machine.turn_off(&mut client).await?;
    }

    Ok(())
}
```
