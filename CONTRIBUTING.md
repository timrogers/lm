# Contributing to lm

Thank you for your interest in contributing to `lm`! This project helps people control their La Marzocco espresso machines from the command line, and we welcome contributions from the community.

## Code of Conduct

By participating in this project, you agree to be respectful and constructive in all interactions. We're building this tool together to make everyone's coffee experience better.

## Getting Started

### Prerequisites

Before you can contribute, you'll need:

- [Rust](https://www.rust-lang.org/tools/install) (latest stable version recommended)
- [Git](https://git-scm.com/) for version control
- A La Marzocco account for testing (optional, but helpful for end-to-end testing)

### Setting Up Your Development Environment

1. **Fork and clone the repository**:
   ```bash
   git clone https://github.com/YOUR_USERNAME/lm.git
   cd lm
   ```

2. **Build the project**:
   ```bash
   cargo build
   ```

3. **Run the tests to ensure everything works**:
   ```bash
   cargo test
   ```

4. **Check that the CLI works**:
   ```bash
   cargo run -- --help
   ```

## Development Workflow

### Making Changes

1. **Create a new branch** for your feature or bug fix:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes** following the coding standards below

3. **Test your changes** thoroughly using the quality checks

### Quality Checks

Before submitting any changes, you **must** run these commands and ensure they all pass:

1. **Run tests** - Always execute the test suite:
   ```bash
   cargo test
   ```

2. **Run Clippy** - Use Clippy to catch common mistakes and improve code quality:
   ```bash
   cargo clippy
   ```

3. **Check code formatting** - Ensure consistent code formatting:
   ```bash
   cargo fmt --check
   ```

4. **Verify the project builds**:
   ```bash
   cargo build
   ```

#### Automated Checks

Our CI pipeline automatically runs these checks on all platforms (Ubuntu, Windows, macOS) when you submit a pull request. Your changes must pass all checks before they can be merged.

### Code Quality Standards

- **Address all Clippy warnings** unless there's a specific reason to ignore them (document why)
- **Maintain consistent formatting** throughout the codebase using `cargo fmt`
- **Ensure all tests pass** before submitting changes
- **Write tests for new functionality** when appropriate
- **Follow existing code patterns** and conventions in the codebase

### Testing Guidelines

- **Unit tests**: Add tests for new functions and modules in the same file using `#[cfg(test)]`
- **Integration tests**: Add end-to-end tests in the `tests/` directory for new CLI commands or major features
- **Documentation tests**: Include examples in doc comments that can be tested with `cargo test`
- **Manual testing**: Test your changes manually with `cargo run` when possible

## Submitting Your Contribution

### Pull Requests

1. **Push your changes** to your fork:
   ```bash
   git push origin feature/your-feature-name
   ```

2. **Create a pull request** on GitHub with:
   - A clear title describing what you've changed
   - A detailed description of the changes and why they're needed
   - Reference to any related issues (e.g., "Fixes #123")
   - Screenshots or examples if you've changed CLI output

3. **Respond to feedback** promptly and make requested changes

### Commit Messages

- Use clear, descriptive commit messages
- Start with a verb in the present tense (e.g., "Add", "Fix", "Update")
- Keep the first line under 50 characters
- Include more details in the body if needed

Example:
```
Add support for machine temperature monitoring

- Implement new `temperature` command
- Add temperature parsing to machine status
- Include temperature in machine status output
- Add tests for temperature functionality
```

## Types of Contributions

We welcome many types of contributions:

- **Bug fixes**: Help us improve reliability
- **New features**: Add functionality that would benefit La Marzocco users
- **Documentation**: Improve README, code comments, or help text
- **Tests**: Increase test coverage or add missing test cases
- **Performance improvements**: Make the CLI faster or more efficient
- **Code quality**: Refactoring, better error handling, or cleanup

## Project Structure

- `src/main.rs` - Main CLI application and command handling
- `src/auth.rs` - Authentication and API client functionality
- `src/types.rs` - Data structures and API response types
- `src/config.rs` - Configuration file management
- `src/lib.rs` - Library exports for programmatic use
- `tests/` - Integration tests
- `.github/` - CI/CD workflows and GitHub configuration

## Getting Help

If you need help or have questions:

- **Check existing issues** on GitHub for similar problems or questions
- **Create a new issue** if you've found a bug or want to discuss a feature
- **Look at the code** - the codebase is well-documented and relatively small
- **Review recent pull requests** to see examples of contributions

## Release Process

Releases are handled by the maintainers and follow this process:

1. Version bump in `Cargo.toml`
2. Git tag creation (e.g., `v0.3.0`)
3. Automated CI builds and publishes to:
   - GitHub Releases (with binaries for multiple platforms)
   - crates.io (Rust package registry)
   - Homebrew tap (for macOS/Linux users)

## License

By contributing to this project, you agree that your contributions will be licensed under the [MIT License](LICENSE.md).

---

Thank you for contributing to `lm`! Your help makes this tool better for everyone who wants to control their La Marzocco machines. ☕️