# Move Generation Design

This document describes the design for implementing legal move generation in the chess engine.

## Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Sliding piece attacks | Magic bitboards | O(1) lookup, ~800KB memory, high performance for competitive bots |
| Legality checking | Pseudo-legal + filter | Simpler to implement, easier to debug with perft |
| Magic numbers | Pre-computed | Zero startup cost, battle-tested values |

## Module Structure

```
crates/chess-engine/src/
├── movegen/
│   ├── mod.rs          # MoveList, generate_moves() entry point
│   ├── attacks.rs      # Attack table initialization and lookup
│   ├── magics.rs       # Magic numbers and magic bitboard tables
│   └── perft.rs        # Perft testing for validation
```

## Attack Tables

### Simple Pieces (Compile-time)

Knight, king, and pawn attacks use precomputed `const` arrays:

```rust
const KNIGHT_ATTACKS: [Bitboard; 64] = { /* computed at compile time */ };
const KING_ATTACKS: [Bitboard; 64] = { /* computed at compile time */ };
const PAWN_ATTACKS: [[Bitboard; 64]; 2] = { /* [Color][Square] */ };
```

### Sliding Pieces (Magic Bitboards)

For bishops and rooks, magic bitboards provide O(1) attack lookup:

1. Mask relevant blocker squares (excluding edges)
2. Extract occupied squares from this mask
3. Multiply by magic number and shift to get table index
4. Look up precomputed attack bitboard

```rust
pub fn bishop_attacks(sq: Square, occupied: Bitboard) -> Bitboard {
    let magic = &BISHOP_MAGICS[sq.index()];
    let blockers = occupied & magic.mask;
    let index = ((blockers.0.wrapping_mul(magic.magic)) >> magic.shift) as usize;
    magic.attacks[index]
}
```

Tables initialized via `OnceLock` on first use (~800KB total).

## Move Generation

### Entry Point

```rust
pub fn generate_moves(position: &Position) -> MoveList {
    let mut moves = MoveList::new();

    generate_pawn_moves(position, &mut moves);
    generate_knight_moves(position, &mut moves);
    generate_bishop_moves(position, &mut moves);
    generate_rook_moves(position, &mut moves);
    generate_queen_moves(position, &mut moves);
    generate_king_moves(position, &mut moves);
    generate_castling_moves(position, &mut moves);

    filter_legal_moves(position, &mut moves);

    moves
}
```

### Piece Move Pattern

```rust
fn generate_knight_moves(position: &Position, moves: &mut MoveList) {
    let us = position.side_to_move;
    let our_pieces = position.colors[us.index()];
    let mut knights = position.pieces_of(Piece::Knight, us);

    while let Some(from) = knights.pop_lsb() {
        let attacks = knight_attacks(from) & !our_pieces;
        for to in attacks {
            moves.push(Move::normal(from, to));
        }
    }
}
```

### Pawn Moves

Pawns require special handling:
- Single and double pushes
- Diagonal captures
- En passant captures
- Promotions (4 piece types)

### Legality Filter

```rust
fn filter_legal_moves(position: &Position, moves: &mut MoveList) {
    moves.retain(|m| {
        let new_pos = make_move(position, *m);
        !is_king_attacked(&new_pos, position.side_to_move)
    });
}
```

## Making Moves

The `make_move()` function creates a new position with:

1. Piece removed from source square
2. Any captured piece removed
3. Piece placed at destination (with promotion if applicable)
4. Special move handling (castling rook, en passant capture)
5. Castling rights updated
6. En passant square set/cleared
7. Clocks updated, side switched

## Testing

### Perft

```rust
pub fn perft(position: &Position, depth: u32) -> u64 {
    if depth == 0 { return 1; }
    let moves = generate_moves(position);
    if depth == 1 { return moves.len() as u64; }

    moves.iter().map(|m| {
        perft(&make_move(position, *m), depth - 1)
    }).sum()
}
```

### Test Vectors

| Position | Depth | Nodes |
|----------|-------|-------|
| Starting | 1 | 20 |
| Starting | 2 | 400 |
| Starting | 3 | 8,902 |
| Starting | 4 | 197,281 |
| Starting | 5 | 4,865,609 |
| Kiwipete | 1 | 48 |
| Kiwipete | 2 | 2,039 |
| Kiwipete | 3 | 97,862 |

Kiwipete FEN: `r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -`

## RuleSet Integration

```rust
impl RuleSet for StandardChess {
    fn generate_moves(&self, position: &Position) -> MoveList {
        movegen::generate_moves(position)
    }

    fn make_move(&self, position: &Position, m: Move) -> Position {
        movegen::make_move(position, m)
    }

    fn is_check(&self, position: &Position) -> bool {
        movegen::is_king_attacked(position, position.side_to_move)
    }

    fn game_result(&self, position: &Position) -> Option<GameResult> {
        let moves = self.generate_moves(position);
        if moves.is_empty() {
            if self.is_check(position) {
                Some(GameResult::Checkmate(position.side_to_move.opposite()))
            } else {
                Some(GameResult::Stalemate)
            }
        } else if position.halfmove_clock >= 100 {
            Some(GameResult::Draw)
        } else {
            None
        }
    }
}
```
