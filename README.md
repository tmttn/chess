# Chess Monorepo

[![CI](https://github.com/tommetten/chess/actions/workflows/ci.yml/badge.svg)](https://github.com/tommetten/chess/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A high-performance chess engine written in Rust, designed for speed, correctness, and extensibility.

## Features

- **Bitboard representation** - Efficient 64-bit board representation for fast move generation
- **WASM support** - Compile to WebAssembly for browser and Node.js usage
- **Extensible rules** - `RuleSet` trait allows implementing chess variants
- **Well-tested** - Unit tests, property-based tests, and perft validation

## Project Structure

```
chess/
├── crates/
│   ├── chess-core/     # Core types: Piece, Square, Move, FEN
│   ├── chess-engine/   # Bitboards, move generation, game state
│   └── chess-wasm/     # WebAssembly bindings
└── packages/           # (Future) TypeScript packages
```

## Quick Start

### Rust

```rust
use chess_engine::{Position, StandardChess};
use chess_engine::rules::RuleSet;

fn main() {
    let position = StandardChess.initial_position();
    let moves = StandardChess.generate_moves(&position);
    println!("Legal moves: {}", moves.len());
}
```

### JavaScript (WASM)

```javascript
import init, { Game } from 'chess-wasm';

await init();

const game = new Game();
console.log(game.toFen());

const moves = game.legalMoves();
game.makeMove("e2e4");
```

## Development

### Prerequisites

- Rust (stable) - Install via [rustup](https://rustup.rs/)
- wasm-pack (for WASM builds) - `cargo install wasm-pack`
- pre-commit (for hooks) - `pip install pre-commit`

### Setup

```bash
# Clone the repository
git clone https://github.com/tommetten/chess.git
cd chess

# Install pre-commit hooks
pre-commit install

# Build all crates
cargo build

# Run tests
cargo test --workspace

# Build WASM
wasm-pack build crates/chess-wasm --target web
```

### Running Checks Locally

```bash
# Format code
cargo fmt --all

# Run lints
cargo clippy --workspace --all-targets -- -D warnings

# Run tests
cargo test --workspace

# Run all pre-commit hooks
pre-commit run --all-files
```

## Roadmap

- [x] Project setup and scaffolding
- [x] Core types (Piece, Square, Move, FEN)
- [x] Position representation with bitboards
- [x] WASM bindings scaffold
- [ ] Attack table generation (magic bitboards)
- [ ] Legal move generation
- [ ] Check/checkmate/stalemate detection
- [ ] Perft testing suite
- [ ] Chess960 variant support
- [ ] CLI tool
- [ ] Web UI
- [ ] Multiplayer server

## Documentation

- [Architecture](docs/ARCHITECTURE.md) - Design decisions and crate structure
- [Contributing](docs/CONTRIBUTING.md) - How to contribute to the project

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
