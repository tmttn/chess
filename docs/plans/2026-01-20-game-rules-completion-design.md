# Chess Engine Rules Completion Design

## Overview

Complete the chess engine's rule coverage by adding a `Game` struct for full game management, SAN notation support, and all FIDE draw conditions.

## Architecture

### Game Struct

```rust
// crates/chess-engine/src/game.rs

pub struct Game {
    /// Current position
    position: Position,

    /// Position history for repetition detection (Zobrist hashes)
    history: Vec<u64>,

    /// Move history with SAN notation
    moves: Vec<GameMove>,

    /// Starting position (for PGN export)
    start_pos: Position,

    /// Rule set being used
    rules: Box<dyn RuleSet>,
}

pub struct GameMove {
    /// The move in internal format
    pub mov: Move,
    /// SAN notation ("Nf3", "O-O", "exd5#")
    pub san: String,
    /// Position hash before the move (for history)
    pub hash_before: u64,
}
```

The `Game` struct owns the full game state. The `history` vector stores Zobrist hashes for efficient repetition detection.

## Draw Detection

### Repetition (threefold/fivefold)

```rust
impl Game {
    fn position_count(&self) -> usize {
        let current_hash = self.position.zobrist_hash();
        self.history.iter().filter(|&&h| h == current_hash).count()
    }

    pub fn can_claim_draw(&self) -> bool {
        self.position_count() >= 3 || self.position.halfmove_clock >= 100
    }

    fn is_automatic_draw(&self) -> bool {
        self.position_count() >= 5 || self.position.halfmove_clock >= 150
    }
}
```

### Insufficient Material (strict FIDE)

Only declare draw when checkmate is literally impossible:
- K vs K
- K+B vs K
- K+N vs K
- K+B vs K+B where both bishops are on same color squares

Any other material combination can theoretically deliver checkmate.

## SAN Notation

| Move type | Format | Example |
|-----------|--------|---------|
| Pawn push | `file + square` | `e4`, `d5` |
| Pawn capture | `file + x + square` | `exd5` |
| Piece move | `piece + square` | `Nf3`, `Bb5` |
| Piece capture | `piece + x + square` | `Nxe5`, `Bxc6` |
| Ambiguous | `piece + file/rank + square` | `Nbd2`, `R1e1` |
| Castling | `O-O` / `O-O-O` | kingside / queenside |
| Promotion | `square + = + piece` | `e8=Q`, `dxc1=N` |
| Check | append `+` | `Bb5+` |
| Checkmate | append `#` | `Qxf7#` |

```rust
impl Game {
    pub fn move_to_san(&self, m: Move) -> String { ... }
    pub fn san_to_move(&self, san: &str) -> Result<Move, SanError> { ... }
    pub fn make_move_san(&mut self, san: &str) -> Result<(), GameError> { ... }
}
```

## Game API

```rust
impl Game {
    // Construction
    pub fn new() -> Self;
    pub fn from_position(pos: Position) -> Self;
    pub fn from_fen(fen: &str) -> Result<Self, FenError>;

    // Making moves
    pub fn make_move(&mut self, m: Move) -> Result<(), GameError>;
    pub fn make_move_san(&mut self, san: &str) -> Result<(), GameError>;

    // Game state queries
    pub fn position(&self) -> &Position;
    pub fn legal_moves(&self) -> MoveList;
    pub fn is_check(&self) -> bool;
    pub fn result(&self) -> Option<GameResult>;
    pub fn is_game_over(&self) -> bool;

    // Draw handling
    pub fn can_claim_draw(&self) -> bool;
    pub fn claim_draw(&mut self) -> Result<(), GameError>;

    // History access
    pub fn move_history(&self) -> &[GameMove];
    pub fn ply_count(&self) -> usize;
    pub fn fullmove_number(&self) -> u32;
}

pub enum GameError {
    IllegalMove,
    InvalidSan(String),
    GameAlreadyOver,
    CannotClaimDraw,
}
```

## File Structure

```
crates/chess-engine/src/
├── lib.rs              # Add: pub mod game, pub mod san
├── game.rs             # New: Game struct, GameMove, GameError
├── san.rs              # New: SAN parsing/generation
├── rules/
│   ├── mod.rs          # Update: add DrawReason enum
│   └── standard.rs     # Update: insufficient_material()
├── movegen/            # Existing (unchanged)
├── bitboard.rs         # Existing (unchanged)
├── position.rs         # Existing (unchanged)
└── zobrist.rs          # Existing (unchanged)
```

## Testing Strategy

### Unit tests

| Component | Test cases |
|-----------|------------|
| SAN generation | All piece types, captures, promotions, castling, check/checkmate |
| SAN parsing | Valid moves, ambiguous moves, invalid input |
| Threefold repetition | Repetition via move sequence |
| Fivefold repetition | Automatic draw at 5 |
| 50/75 move rule | Claimable at 100, automatic at 150 half-moves |
| Insufficient material | K vs K, K+B vs K, K+N vs K, K+B vs K+B same color |
| Game flow | New game, make moves, claim draw, game over states |

### Integration tests

- Famous games replayed via SAN
- Known draw positions
- Edge cases from real tournament play

### Coverage target

Maintain 80%+ as per project rules.
