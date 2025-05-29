# LM - La Marzocco CLI

A command-line interface for controlling La Marzocco espresso machines remotely through their cloud service.

## Features

- **List machines**: View all machines connected to your account with model, name, location, and status
- **Turn on machines**: Remotely turn on your espresso machine  
- **Turn off machines**: Put your machine into standby mode
- **Environment variable support**: Store credentials securely using environment variables

## Installation

Make sure you have Rust installed, then build the project:

```bash
cargo build --release
```

The binary will be available at `target/release/lm`.

## Usage

### Authentication

You can provide credentials in two ways:

1. **Command line arguments**:
   ```bash
   lm --username your@email.com --password yourpassword machines
   ```

2. **Environment variables** (recommended):
   ```bash
   export LM_USERNAME="your@email.com"
   export LM_PASSWORD="yourpassword"
   lm machines
   ```

### Commands

#### List machines
```bash
lm machines
```

Shows all machines connected to your account:
```
Model                Name                           Location             Serial               Status    
----------------------------------------------------------------------------------------------------
GS3 AV              Kitchen Machine                Home                 GS01234              On        
Linea Mini          Office Espresso                Work                 LM56789              Standby   
GS3 MP              Garage Machine                 Garage               GS98765              Unavailable
```

**Status meanings:**
- **On** - Machine is connected and powered on (brewing mode)
- **Standby** - Machine is connected but in standby mode
- **Unavailable** - Machine is not currently connected to the network

#### Turn on a machine
```bash
# If you have only one machine
lm2 on

# If you have multiple machines, specify the serial number
lm2 on --serial ABC123
```

#### Turn off a machine (standby mode)
```bash
# If you have only one machine
lm2 off

# If you have multiple machines, specify the serial number
lm2 off --serial ABC123
```

### Help

Get help for any command:
```bash
lm2 --help
lm2 on --help
lm2 off --help
lm2 machines --help
```

## API

This CLI uses the La Marzocco customer app API at `https://lion.lamarzocco.io/api/customer-app`.

The implementation is inspired by the [pylamarzocco](https://github.com/zweckj/pylamarzocco) Python library.

## License

MIT License
