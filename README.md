# lm

💡☕ Control your La Marzocco espresso machine from the command line

---

## Features

With this tool, you can:

- Turn your machine on
- Switch your machine back into standby
- Monitor the status of your machine

## Installation

### macOS or Linux via [Homebrew](https://brew.sh/)

1. Install the latest version by running `brew tap timrogers/tap && brew install lm`.
1. Run `lm --help` to check that everything is working and see the available commands.

### macOS, Linux or Windows via [Cargo](https://doc.rust-lang.org/cargo/), Rust's package manager

1. Install [Rust](https://www.rust-lang.org/tools/install) on your machine, if it isn't already installed.
1. Install the `lm` crate by running `cargo install lm`.
1. Run `lm --help` to check that everything is working and see the available commands.

### macOS, Linux or Windows via direct binary download

1. Download the [latest release](https://github.com/timrogers/lm/releases/latest) for your platform. macOS, Linux and Windows devices are supported.
2. Add the binary to `$PATH`, so you can execute it from your shell. For the best experience, call it `lm` on macOS and Linux, and `lm.exe` on Windows.
3. Run `lm --help` to check that everything is working and see the available commands.

## Usage

### From the command line

#### Logging in to your La Marzocco account

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

#### Viewing the status of your machine(s) 

```bash
lm machines
```

You'll see all of the machines connected to your account, with their status:

```
Model                Name                           Location             Serial               Status    
----------------------------------------------------------------------------------------------------
GS3 AV              Kitchen Machine                Home                 GS01234              On        
Linea Mini          Office Espresso                Work                 LM56789              Standby   
GS3 MP              Garage Machine                 Garage               GS98765              Unavailable
```

#### Turning on a machine

```bash
# Turn your one and only machine on
lm on

# Turn your machine on, wait until the coffee boiler is ready to go, then exit and trigger a notification
lm on --wait

# Turn on a specific machine, specified by serial number
lm on --serial ABC123
```

#### Turning off a machine (standby mode)

```bash
# Switch your one and only machine into standby
lm off

# Switch a specific machine into standby mode, specified by serial number
lm off --serial ABC123
```

### From a Rust application

The `lm` crate includes functions for interacting with La Marzocco espresso machines from your Rust applications.

To see the full API, check out the documentation on [Docs.rs](https://docs.rs/lm/) or read through [`src/lib.rs`](src/lib.rs).