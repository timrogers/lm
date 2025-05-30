# lm

A command-line interface (CLI) for controlling La Marzocco espresso machines

## Features

- **🔑 Persistent login**: Log in once and store credentials securely in `~/.lm.yml`
- **📜 List machines**: View all machines connected to your account with model, name, location, and status
- **🔋 Turn on machines**: Remotely turn on your espresso machine  
- **😴 Turn off machines**: Put your machine into standby mode
- **🔃 Automatic token refresh**: Access tokens are automatically refreshed as needed

## Installation

Make sure you have Rust installed, then build the project:

```bash
cargo build --release
```

The binary will be available at `target/release/lm`.

## Usage

### Authentication

The recommended way to use the CLI is with the persistent login system:

1. **Login once and store credentials** (recommended):
   ```bash
   lm login
   # You'll be prompted for username and password
   # Credentials are securely stored in ~/.lm.yml
   
   # Now you can use any command without providing credentials again
   lm machines
   lm on
   lm off
   ```

   You can also provide credentials directly to the login command:
   ```bash
   lm login --username your@email.com --password yourpassword
   ```

2. **Logout to clear stored credentials**:
   ```bash
   lm logout
   ```

3. **Alternative: Command line arguments** (not recommended):
   ```bash
   lm --username your@email.com --password yourpassword machines
   ```

4. **Alternative: Environment variables**:
   ```bash
   export LM_USERNAME="your@email.com"
   export LM_PASSWORD="yourpassword"
   lm machines
   ```

**Note**: The CLI will automatically refresh access tokens as needed. If stored credentials become invalid, you'll be prompted to run `lm login` again.

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
lm on

# If you have multiple machines, specify the serial number
lm on --serial ABC123
```

#### Turn off a machine (standby mode)
```bash
# If you have only one machine
lm off

# If you have multiple machines, specify the serial number
lm off --serial ABC123
```

### Help

Get help for any command:
```bash
lm --help
lm on --help
lm off --help
lm machines --help
lm login --help
lm logout --help
```