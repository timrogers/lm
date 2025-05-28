# lm
A CLI and Rust library for controlling La Marzocco Home espresso machines

## Installation

```
cargo install --path .
```

## Usage

### Initial Setup

Before using the CLI, you need to set up your La Marzocco Home account credentials:

```
lm setup --email your.email@example.com --password your-password
```

This will store your credentials in a `~/.lm.yml` file.

### Checking Machine Status

To check the status of your espresso machine:

```
lm status
```

If you have multiple machines, you can specify the serial number:

```
lm status --serial-number YOUR_SERIAL_NUMBER
```

### Turning On Your Machine

To turn on your espresso machine:

```
lm on
```

Or with a specific serial number:

```
lm on --serial-number YOUR_SERIAL_NUMBER
```

### Turning Off Your Machine

To turn off your espresso machine:

```
lm off
```

Or with a specific serial number:

```
lm off --serial-number YOUR_SERIAL_NUMBER
```

## Using as a Library

You can also use `lm` as a Rust library in your own projects:

```rust
use lm::LaMarzoccoClient;

#[tokio::main]
async fn main() {
    let client = LaMarzoccoClient::new();
    
    // List all machines
    let machines = client.list_machines().await.unwrap();
    
    // Turn on a machine
    client.turn_on_machine(&machines[0].serial_number).await.unwrap();
}
```
