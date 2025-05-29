# GitHub Copilot Instructions

When working on this repository, please follow these guidelines:

## Testing and Quality Checks

1. **Run tests** - Always execute the test suite before committing changes:
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

## Development Workflow

- Run these commands frequently during development to catch issues early
- Fix any warnings or errors before finalizing changes
- Consider running `cargo build` to ensure the code compiles successfully
- Use `cargo run` to test functionality manually when appropriate

## Code Quality Standards

- Address all Clippy warnings unless there's a specific reason to ignore them
- Maintain consistent formatting throughout the codebase
- Ensure all tests pass before submitting changes
- Write tests for new functionality when appropriate