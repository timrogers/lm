// CLI integration tests
// These test the actual command-line interface using the compiled binary

use std::process::Command;

const CLI_BINARY: &str = env!("CARGO_BIN_EXE_lm");

#[tokio::test]
async fn test_cli_machines_command_no_credentials() {
    // Test that the CLI fails gracefully when no credentials are provided
    let output = Command::new(CLI_BINARY)
        .arg("machines")
        .env_remove("LM_USERNAME")
        .env_remove("LM_PASSWORD")
        .output()
        .expect("Failed to execute CLI");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("You don't seem to be logged in."));
}

#[tokio::test]
async fn test_cli_help_command() {
    // Test that help command works
    let output = Command::new(CLI_BINARY)
        .arg("--help")
        .output()
        .expect("Failed to execute CLI");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("A CLI for controlling La Marzocco espresso machines"));
    assert!(stdout.contains("login"));
    assert!(stdout.contains("logout"));
    assert!(stdout.contains("machines"));
    assert!(stdout.contains("on"));
    assert!(stdout.contains("off"));
}

#[tokio::test]
async fn test_cli_version_command() {
    // Test that version command works
    let output = Command::new(CLI_BINARY)
        .arg("--version")
        .output()
        .expect("Failed to execute CLI");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("lm"));
}

#[tokio::test]
async fn test_cli_invalid_command() {
    // Test that invalid commands are handled properly
    let output = Command::new(CLI_BINARY)
        .arg("invalid-command")
        .output()
        .expect("Failed to execute CLI");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("error:") || stderr.contains("unrecognized"));
}

#[tokio::test]
async fn test_cli_on_command_with_wait_no_credentials() {
    // Test that the CLI fails gracefully when using --wait without credentials
    let output = Command::new(CLI_BINARY)
        .args(["on", "--wait"])
        .env_remove("LM_USERNAME")
        .env_remove("LM_PASSWORD")
        .output()
        .expect("Failed to execute CLI");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("You don't seem to be logged in."));
}

#[tokio::test]
async fn test_cli_on_command_help_includes_wait() {
    // Test that the on command help includes the --wait flag
    let output = Command::new(CLI_BINARY)
        .args(["on", "--help"])
        .output()
        .expect("Failed to execute CLI");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--wait"));
    assert!(stdout.contains("Wait for the machine to be ready to brew before exiting, and trigger a notification when ready"));
}

#[tokio::test]
async fn test_cli_login_command_help() {
    // Test that the login command help works and shows correct options
    let output = Command::new(CLI_BINARY)
        .args(["login", "--help"])
        .output()
        .expect("Failed to execute CLI");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Log in to your La Marzocco account and store credentials for future use")
    );
    assert!(stdout.contains("--username"));
    assert!(stdout.contains("--password"));
}

#[tokio::test]
async fn test_cli_logout_command() {
    // Test that the logout command works (doesn't matter if no credentials are stored)
    let output = Command::new(CLI_BINARY)
        .arg("logout")
        .output()
        .expect("Failed to execute CLI");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Logged out successfully"));
}

#[tokio::test]
async fn test_cli_verbose_flag_in_help() {
    // Test that the --verbose flag appears in the help output
    let output = Command::new(CLI_BINARY)
        .arg("--help")
        .output()
        .expect("Failed to execute CLI");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--verbose"));
    assert!(stdout.contains("Enable verbose logging"));
}

#[tokio::test]
async fn test_cli_verbose_flag_functionality() {
    // Test that the --verbose flag works and doesn't break basic functionality
    let output = Command::new(CLI_BINARY)
        .args(["--verbose", "logout"])
        .output()
        .expect("Failed to execute CLI");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Logged out successfully"));
}

#[tokio::test]
async fn test_cli_rejects_config_without_version() {
    // Test that authenticated commands reject config files without version field
    use std::fs;
    use tempfile::TempDir;

    // Create a temporary directory to simulate a user's home directory
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join(".lm.yml");

    // Create a config file without version field (simulating old CLI version)
    let old_config = r#"
username: test@example.com
access_token: fake_access_token
refresh_token: fake_refresh_token
"#;

    fs::write(&config_path, old_config).expect("Failed to write test config");

    // Test that machines command rejects the config without version
    let mut cmd = Command::new(CLI_BINARY);
    cmd.arg("machines");

    // Set the appropriate home directory environment variable based on platform
    #[cfg(windows)]
    cmd.env("USERPROFILE", temp_dir.path());
    #[cfg(not(windows))]
    cmd.env("HOME", temp_dir.path());

    let output = cmd.output().expect("Failed to execute CLI");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("older version of the CLI"));
    assert!(stderr.contains("Please run 'lm login' again"));
}

// Note: We could add more comprehensive CLI tests that actually hit mocked endpoints,
// but that would require modifying the CLI to accept a custom base URL parameter,
// which might not be worth the complexity for this project.
