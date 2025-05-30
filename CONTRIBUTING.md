# Contributing to LM

Thank you for your interest in contributing to LM! This guide will help you get started with contributing to this La Marzocco espresso machine CLI tool.

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable version)
- Git

### Development Setup

1. Fork the repository
2. Clone your fork:
   ```bash
   git clone https://github.com/your-username/lm.git
   cd lm
   ```
3. Build the project:
   ```bash
   cargo build
   ```
4. Run tests to ensure everything works:
   ```bash
   cargo test
   ```

## Development Guidelines

### Code Style

- Follow standard Rust formatting with `rustfmt`:
  ```bash
  cargo fmt
  ```
- Use `clippy` for linting:
  ```bash
  cargo clippy
  ```
- All code should compile without warnings

### Testing

- Write tests for new functionality
- Ensure all tests pass before submitting:
  ```bash
  cargo test
  ```
- Integration tests are located in the `tests/` directory
- Use the existing test fixtures in `tests/fixtures/` for consistent test data

### Code Organization

- Keep the main CLI logic in `src/main.rs`
- Business logic should be in appropriate modules under `src/`
- Follow the existing module structure:
  - `auth.rs` - Authentication logic
  - `client.rs` - API client implementation
  - `config.rs` - Configuration management
  - `types.rs` - Data structures and types

### Dependencies

- Prefer well-established crates with good maintenance
- Add new dependencies sparingly and justify their inclusion
- Update `Cargo.toml` with appropriate version constraints

## Pull Request Process

1. Create a feature branch from `main`:
   ```bash
   git checkout -b feature/your-feature-name
   ```
2. Make your changes following the guidelines above
3. Add or update tests as needed
4. Ensure your code passes all checks:
   ```bash
   cargo fmt --check
   cargo clippy -- -D warnings
   cargo test
   ```
5. Commit your changes with clear, descriptive messages
6. Push to your fork and create a pull request

### Pull Request Guidelines

- Provide a clear description of what your PR does
- Reference any related issues
- Include tests for new functionality
- Update documentation if needed
- Keep PRs focused on a single feature or fix

## Reporting Issues

When reporting issues, please include:

- Your operating system and version
- Rust version (`rustc --version`)
- Steps to reproduce the issue
- Expected vs actual behavior
- Any relevant error messages or logs

## API Integration

This project integrates with the La Marzocco customer app API. When working on API-related features:

- Use the existing `ApiClient` in `src/client.rs`
- Add appropriate error handling for network requests
- Follow the patterns established in the existing API calls
- Consider rate limiting and authentication token refresh

## Security Considerations

- Never commit credentials or sensitive data
- Be careful with credential storage and handling
- Review security implications of any changes to authentication logic

## Getting Help

- Check existing issues and discussions
- Look at the codebase for examples
- Feel free to ask questions in issue discussions

## Code of Conduct

Be respectful and constructive in all interactions. We want this to be a welcoming environment for all contributors.

Thank you for contributing to LM!