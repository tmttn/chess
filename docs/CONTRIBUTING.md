# Contributing

Thank you for your interest in contributing to the chess monorepo!

## Getting Started

### Prerequisites

- **Rust** (stable) - Install via [rustup](https://rustup.rs/)
- **wasm-pack** - For WASM builds: `cargo install wasm-pack`
- **pre-commit** - For git hooks: `pip install pre-commit`

### Setup

```bash
# Clone the repository
git clone https://github.com/tommetten/chess.git
cd chess

# Install pre-commit hooks
pre-commit install

# Verify everything builds
cargo build --workspace

# Run tests
cargo test --workspace
```

## Development Workflow

### Before You Start

1. Check existing issues to see if your contribution is already being worked on
2. For larger changes, open an issue first to discuss the approach

### Making Changes

1. Create a feature branch from `main`:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. Make your changes, following the code style guidelines below

3. Run all checks locally:
   ```bash
   # Format code
   cargo fmt --all

   # Run lints
   cargo clippy --workspace --all-targets -- -D warnings

   # Run tests
   cargo test --workspace

   # Or run all pre-commit hooks
   pre-commit run --all-files
   ```

4. Commit your changes with a descriptive message:
   ```bash
   git commit -m "Add feature X that does Y"
   ```

5. Push and open a pull request

### Pull Request Guidelines

- Keep PRs focused on a single change
- Include tests for new functionality
- Update documentation if needed
- Ensure CI passes before requesting review

## Code Style

### Rust

- Follow standard Rust conventions
- Use `cargo fmt` for formatting
- Address all `cargo clippy` warnings
- Write doc comments for public APIs

```rust
/// Generates all legal moves for the given position.
///
/// # Arguments
///
/// * `position` - The current game position
///
/// # Returns
///
/// A list of all legal moves
pub fn generate_moves(&self, position: &Position) -> MoveList {
    // Implementation
}
```

### Commit Messages

- Use the imperative mood ("Add feature" not "Added feature")
- Keep the first line under 72 characters
- Reference issues when applicable ("Fix #123")

### Testing

- Write unit tests for new functionality
- Add integration tests for complex features
- Use property-based tests for invariants

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_specific_behavior() {
        // Arrange
        let position = Position::startpos();

        // Act
        let result = some_function(&position);

        // Assert
        assert_eq!(result, expected_value);
    }
}
```

## Project Structure

```
chess/
├── crates/
│   ├── chess-core/     # Core types (Piece, Square, Move)
│   ├── chess-engine/   # Engine logic (bitboards, move gen)
│   └── chess-wasm/     # WASM bindings
├── docs/               # Documentation
└── packages/           # Future TypeScript packages
```

### Adding to chess-core

`chess-core` contains stable types. Changes here should be:
- Backwards compatible when possible
- Minimal and focused
- Well documented

### Adding to chess-engine

`chess-engine` contains the main logic. When adding features:
- Consider performance implications
- Add appropriate tests
- Update the `RuleSet` trait if needed

### Adding to chess-wasm

`chess-wasm` is a thin wrapper. Keep it minimal:
- Only expose what's needed for common use cases
- Use string-based interfaces for simplicity
- Keep the bundle size small

## Running Benchmarks

```bash
# Run benchmarks (when implemented)
cargo bench --workspace
```

## Reporting Issues

When reporting bugs, please include:

1. A clear description of the problem
2. Steps to reproduce
3. Expected vs actual behavior
4. Rust version (`rustc --version`)
5. Operating system

## Getting Help

- Open an issue for questions
- Check existing documentation
- Look at test files for usage examples

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
