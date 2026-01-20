//! Zobrist hashing for position identification.
//!
//! Zobrist hashing creates a unique hash for each chess position by XORing
//! random numbers associated with:
//! - Each piece on each square (12 pieces × 64 squares = 768 values)
//! - Side to move
//! - Castling rights (4 values)
//! - En passant file (8 values)
//!
//! This allows efficient incremental updates when making moves.

// Allow dead code - this module is scaffolded for future use in move generation
#![allow(dead_code)]

use chess_core::{Color, Piece, Square};

/// Zobrist hash keys.
///
/// Generated using a fixed seed for reproducibility.
pub struct ZobristKeys {
    /// Keys for pieces: [piece][color][square]
    pub pieces: [[[u64; 64]; 2]; 6],
    /// Key for black to move (XOR when black to move).
    pub black_to_move: u64,
    /// Keys for castling rights.
    pub castling: [u64; 4],
    /// Keys for en passant file.
    pub en_passant: [u64; 8],
}

impl ZobristKeys {
    /// Initializes Zobrist keys using a simple PRNG.
    ///
    /// Uses a fixed seed for reproducibility across runs.
    pub const fn new() -> Self {
        // Simple xorshift64 PRNG for const initialization
        const fn next_random(state: u64) -> (u64, u64) {
            let mut x = state;
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            (x, x)
        }

        let mut state = 0x9E3779B97F4A7C15u64; // Golden ratio seed
        let mut pieces = [[[0u64; 64]; 2]; 6];
        let mut castling = [0u64; 4];
        let mut en_passant = [0u64; 8];

        // Initialize piece keys
        let mut piece = 0;
        while piece < 6 {
            let mut color = 0;
            while color < 2 {
                let mut square = 0;
                while square < 64 {
                    let (new_state, value) = next_random(state);
                    state = new_state;
                    pieces[piece][color][square] = value;
                    square += 1;
                }
                color += 1;
            }
            piece += 1;
        }

        // Initialize black to move key
        let (new_state, black_to_move) = next_random(state);
        state = new_state;

        // Initialize castling keys
        let mut i = 0;
        while i < 4 {
            let (new_state, value) = next_random(state);
            state = new_state;
            castling[i] = value;
            i += 1;
        }

        // Initialize en passant keys
        let mut i = 0;
        while i < 8 {
            let (new_state, value) = next_random(state);
            state = new_state;
            en_passant[i] = value;
            i += 1;
        }

        ZobristKeys {
            pieces,
            black_to_move,
            castling,
            en_passant,
        }
    }

    /// Returns the key for a piece on a square.
    #[inline]
    pub const fn piece_key(&self, piece: Piece, color: Color, square: Square) -> u64 {
        self.pieces[piece.index()][color.index()][square.index() as usize]
    }

    /// Returns the key for a castling right (0-3).
    #[inline]
    pub const fn castling_key(&self, right: usize) -> u64 {
        self.castling[right]
    }

    /// Returns the key for an en passant file (0-7).
    #[inline]
    pub const fn en_passant_key(&self, file: usize) -> u64 {
        self.en_passant[file]
    }
}

/// Global Zobrist keys (initialized at compile time).
pub static ZOBRIST: ZobristKeys = ZobristKeys::new();

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zobrist_keys_are_nonzero() {
        // Most keys should be nonzero (statistically almost certain)
        assert_ne!(ZOBRIST.black_to_move, 0);
        assert_ne!(ZOBRIST.pieces[0][0][0], 0);
        assert_ne!(ZOBRIST.castling[0], 0);
    }

    #[test]
    fn zobrist_keys_are_unique() {
        // Check that piece keys are unique (sample check)
        let key1 = ZOBRIST.piece_key(Piece::Pawn, Color::White, Square::A1);
        let key2 = ZOBRIST.piece_key(Piece::Pawn, Color::White, Square::B1);
        let key3 = ZOBRIST.piece_key(Piece::Pawn, Color::Black, Square::A1);
        let key4 = ZOBRIST.piece_key(Piece::Knight, Color::White, Square::A1);

        assert_ne!(key1, key2);
        assert_ne!(key1, key3);
        assert_ne!(key1, key4);
    }
}
