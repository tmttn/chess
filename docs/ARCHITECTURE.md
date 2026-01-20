# Architecture

This document describes the high-level architecture and design decisions of the chess monorepo.

## Overview

The project is structured as a Rust workspace with multiple crates:

```
chess/
├── crates/
│   ├── chess-core/     # Stable, minimal types
│   ├── chess-engine/   # Move generation, game logic
│   └── chess-wasm/     # WASM bindings
└── packages/           # Future TypeScript packages
```

## Crate Responsibilities

### chess-core

The foundation crate containing stable types shared across all other crates:

- `Piece` - Enum for the six piece types
- `Color` - Enum for White/Black
- `Square` - Board coordinate (0-63)
- `File` / `Rank` - Board file (a-h) and rank (1-8)
- `Move` - Compact move representation (16 bits)
- `FenParser` - FEN string parsing and serialization

This crate has no dependencies on the engine implementation, allowing it to be used independently or by future crates.

### chess-engine

The core engine with bitboard-based position representation:

- `Bitboard` - 64-bit board representation with efficient operations
- `Position` - Full game state (piece positions, castling, en passant, etc.)
- `MoveList` - Fixed-size array for legal moves (no heap allocation)
- `RuleSet` - Trait for implementing chess variants
- `StandardChess` - Standard FIDE rules implementation
- Zobrist hashing for position identification

### chess-wasm

Thin wrapper exposing the engine to JavaScript:

- `Game` struct with methods callable from JS
- FEN import/export
- Move generation and validation
- Game state queries

## Design Decisions

### Bitboard Representation

We use bitboards (64-bit integers where each bit represents a square) for several reasons:

1. **Parallel operations** - A single bitwise AND/OR/XOR operates on all 64 squares simultaneously
2. **Cache efficiency** - Piece positions fit in CPU registers
3. **Pattern matching** - Attack patterns, pins, and checks can be computed with bitwise operations

The layout uses little-endian rank-file mapping:
- Bit 0 = a1, Bit 1 = b1, ..., Bit 63 = h8

### Magic Bitboards

For sliding pieces (bishops, rooks, queens), we use magic bitboards:

1. Precompute attack tables for all blocker configurations
2. Use "magic" multiplication to hash blocker patterns to table indices
3. Achieve O(1) attack generation for sliding pieces

This trades ~800KB of memory for constant-time slider attack generation.

### Move Encoding

Moves are packed into 16 bits:
- Bits 0-5: From square (0-63)
- Bits 6-11: To square (0-63)
- Bits 12-15: Flags (normal, castling, en passant, promotion type)

This compact representation is crucial for move ordering and transposition tables in search.

### RuleSet Trait

The `RuleSet` trait abstracts game rules from the engine:

```rust
pub trait RuleSet {
    fn initial_position(&self) -> Position;
    fn generate_moves(&self, position: &Position) -> MoveList;
    fn is_legal(&self, position: &Position, m: Move) -> bool;
    fn make_move(&self, position: &Position, m: Move) -> Position;
    fn is_check(&self, position: &Position) -> bool;
    fn game_result(&self, position: &Position) -> Option<GameResult>;
}
```

This enables:
- Chess960 (Fischer Random) support
- Other variants (3-check, King of the Hill, etc.)
- Custom rule sets for testing

### WASM Design

The WASM bindings are intentionally thin:

1. **No game logic in WASM crate** - Just type conversions
2. **Minimal API surface** - Only what's needed for common use cases
3. **String-based interface** - FEN and UCI for interoperability

This keeps the WASM bundle small and the API stable.

## Performance Considerations

### Memory Layout

- `Position` struct uses array-based storage for predictable layout
- `MoveList` uses a fixed-size array (256 moves) to avoid heap allocation
- Zobrist keys are computed at compile time

### Hot Paths

Move generation is the most performance-critical code:

1. Piece bitboards are iterated using `pop_lsb()` (extract lowest set bit)
2. Attack generation uses precomputed tables
3. Legal move filtering checks pins and checks efficiently

### Future Optimizations

When implementing search:

- Move ordering (captures first, killer moves, etc.)
- Transposition tables using Zobrist hashes
- Lazy move generation (generate captures first)

## Testing Strategy

### Unit Tests

Each module contains inline `#[cfg(test)]` tests for isolated functionality.

### Integration Tests

`crates/chess-engine/tests/` contains full game scenario tests.

### Property-Based Tests

Using `proptest` to verify invariants:
- FEN roundtrip (parse → serialize → parse equals original)
- Move generation produces legal moves only
- Position hashes are consistent

### Perft Tests

Perft (performance test) counts nodes at each depth to verify move generator correctness:

```
Position: startpos
Depth 1: 20 nodes
Depth 2: 400 nodes
Depth 3: 8902 nodes
Depth 4: 197281 nodes
Depth 5: 4865609 nodes
```

These are verified against known-correct values.

## Future Extensions

### Chess960

The `RuleSet` trait allows Chess960 support by:
1. Different `initial_position()` with shuffled back rank
2. Modified castling rules in `make_move()` and `generate_moves()`

### Search Engine

Future bot implementation will add:
- Alpha-beta search with iterative deepening
- Evaluation function
- Opening book
- Endgame tablebases

### Multiplayer

WebSocket server for online play:
- Game state synchronization
- Move validation on server
- Spectator mode
