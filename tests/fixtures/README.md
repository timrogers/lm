# Test Fixtures

This directory contains JSON fixture files with real API responses captured from the La Marzocco API. These fixtures are used in the test suite to provide realistic test data without requiring actual API calls.

## Files

- **`auth_success.json`** - Successful authentication response with JWT token
- **`auth_failure.json`** - Failed authentication response (401 error)
- **`machines.json`** - List of user's machines (Linea Micra + GS3 AV)
- **`machine_status_on.json`** - Machine status when powered on and boiler ready
- **`machine_status_warming.json`** - Machine status when powered on but boiler still heating
- **`machine_status_standby.json`** - Machine status in standby mode
- **`machine_status_no_widget.json`** - Edge case: status response without CMMachineStatus widget
- **`machine_command_success.json`** - Successful command execution response
- **`machine_command_error.json`** - Error response for invalid commands

## Usage

These fixtures are embedded at compile time via `include_str!` directly in test files throughout the test suite to mock HTTP responses with realistic data.

## Data Source

All responses were captured from the actual La Marzocco Lion API using authenticated requests with real machine data (serial numbers anonymized where appropriate).
