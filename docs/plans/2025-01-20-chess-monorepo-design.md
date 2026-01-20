# Chess Monorepo Design

**Date**: 2025-01-20
**Status**: Approved

## Overview

A Rust-based chess monorepo designed for high performance and web compatibility. The core engine uses bitboard representation for fast move generation, compiles to WASM for browser use, and is architected to support chess variants.

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Language | Rust + WASM | Maximum performance, memory safety, runs natively and in browsers |
| Performance | Bitboards, magic bitboards | Required for competitive bot development (millions of positions/sec) |
| Extensibility | RuleSet trait | Clean abstraction for chess variants without engine changes |
| Monorepo tooling | Cargo workspaces | Native Rust support, simple and effective |
| CI/CD | GitHub Actions | Industry standard, good Rust/WASM support |
| Quality gates | Pre-commit + CI | Catch issues locally first, CI as safety net |
| License | MIT | Maximum permissive, adoption-friendly |

## Repository Structure

```
chess/
├── Cargo.toml              # Workspace root
├── README.md
├── LICENSE
├── CHANGELOG.md
├── .pre-commit-config.yaml
├── rust-toolchain.toml
├── .github/
│   └── workflows/
│       ├── ci.yml
│       ├── release.yml
│       └── wasm.yml
├── docs/
│   ├── ARCHITECTURE.md
│   ├── CONTRIBUTING.md
│   └── plans/
├── crates/
│   ├── chess-core/         # Stable, minimal types
│   ├── chess-engine/       # Bitboards, move generation, game state
│   ├── chess-wasm/         # WASM bindings
│   └── chess-cli/          # (Future) CLI tool
└── packages/               # (Future) TypeScript packages
    ├── chess-ui/
    └── chess-server/
```

## Crate Architecture

### chess-core

Minimal, stable types shared across all crates:

- `Piece` - Enum: Pawn, Knight, Bishop, Rook, Queen, King
- `Color` - Enum: White, Black
- `Square` - 0-63 index with File/Rank accessors
- `Move` - From/to squares with flags (promotion, castling, en passant)
- FEN parsing and serialization

### chess-engine

The heavy lifting:

- `Bitboard` - `u64` wrapper with chess-specific operations
- `attacks` - Precomputed attack tables, magic bitboards for sliders
- `Position` - Full game state (12 piece bitboards, castling rights, en passant, halfmove clock)
- `movegen` - Legal move generation
- `zobrist` - Position hashing for transposition tables
- `RuleSet` trait - Extensibility point for variants

```rust
pub trait RuleSet {
    fn generate_moves(&self, position: &Position) -> MoveList;
    fn is_game_over(&self, position: &Position) -> Option<GameResult>;
    fn initial_position(&self) -> Position;
}
```

### chess-wasm

Thin adapter layer for JavaScript:

```rust
#[wasm_bindgen]
pub struct Game { /* wraps Position + RuleSet */ }

#[wasm_bindgen]
impl Game {
    pub fn new() -> Game;
    pub fn from_fen(fen: &str) -> Result<Game, JsError>;
    pub fn to_fen(&self) -> String;
    pub fn legal_moves(&self) -> JsValue;
    pub fn make_move(&mut self, uci: &str) -> Result<(), JsError>;
    pub fn is_check(&self) -> bool;
    pub fn is_game_over(&self) -> bool;
    pub fn result(&self) -> Option<String>;
}
```

Target size: under 100KB gzipped.

## Quality Gates

### Pre-commit Hooks

Run before every commit:

1. `cargo fmt --check` - Consistent formatting
2. `cargo clippy -- -D warnings` - Lints as errors
3. `cargo test` - All tests pass

### CI Pipeline (every PR)

1. Format check
2. Clippy lints
3. Tests (debug + release)
4. Coverage report
5. WASM build verification

### Release Pipeline (on version tags)

1. Build release binaries (multi-platform)
2. Build WASM package
3. Publish to crates.io
4. Create GitHub release

## Testing Strategy

- **Unit tests**: Inline `#[cfg(test)]` modules in each crate
- **Integration tests**: `crates/chess-engine/tests/` for full game scenarios
- **Property-based tests**: `proptest` for move generation correctness
- **Perft tests**: Verify move generator against known position node counts
- **Coverage**: `cargo-llvm-cov`, tracked in CI

## Initial Deliverables

1. Repository skeleton with all directories
2. Cargo workspace configuration
3. `chess-core` with basic types
4. `chess-engine` scaffolded with RuleSet trait
5. `chess-wasm` placeholder
6. Pre-commit hooks configured
7. CI/CD workflows
8. Documentation (README, ARCHITECTURE, CONTRIBUTING)

## Future Work

- Bitboard implementation and magic attack tables
- Move generation for all piece types
- Check/checkmate/stalemate detection
- Perft test suite
- WASM optimization
- Chess960 variant
- CLI tool
- TypeScript UI package
- Multiplayer server
