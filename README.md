# lm
A CLI and Rust library for controlling La Marzocco Home espresso machines

## Installation

```shell
cargo install lm
```

## Usage

### Logging in

Before using the CLI, you need to log in with your La Marzocco Home account:

```shell
lm login --email "your.email@example.com" --password "your-password"
```

This will store your credentials and tokens securely in `~/.lm.yml`.

### Viewing machine status

To check the status of your connected machines:

```shell
lm status
```

This will display information about each machine, including:
- Machine name and model
- Serial number
- Power status (on/off/brewing)

### Turning a machine on

To turn on your espresso machine:

```shell
lm on
```

If you have multiple machines, you can specify which one to control:

```shell
lm on --serial "YOUR_MACHINE_SERIAL"
```

### Turning a machine off

To turn off your espresso machine:

```shell
lm off
```

Or with a specific serial number:

```shell
lm off --serial "YOUR_MACHINE_SERIAL"
```

## Library Usage

The package also provides a Rust library that can be used in your own projects:

```rust
use lm::{LaMarzoccoClient, create_initial_config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create initial config
    create_initial_config("your.email@example.com".to_string(), "your-password".to_string())?;
    
    // Create client from saved config
    let mut client = LaMarzoccoClient::from_config().await?;
    
    // List machines
    let machines = client.list_machines().await?;
    for machine in &machines {
        println!("Machine: {} ({})", machine.name, machine.serial_number);
    }
    
    // Turn on a machine (using the first one found)
    if !machines.is_empty() {
        client.turn_on(&machines[0].serial_number).await?;
    }
    
    Ok(())
}
```
