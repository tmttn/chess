//! Magic bitboard tables for sliding piece attack generation.
//!
//! Magic bitboards use a perfect hashing technique to map blocker configurations
//! to precomputed attack bitboards in O(1) time.

use crate::Bitboard;
use chess_core::Square;
use std::sync::OnceLock;

/// Magic entry for a single square.
#[derive(Clone)]
pub struct Magic {
    /// Mask of relevant blocker squares (excludes edges).
    pub mask: Bitboard,
    /// The magic number for this square.
    pub magic: u64,
    /// Right shift amount (64 - number of bits in mask).
    pub shift: u8,
    /// Offset into the attack table.
    pub offset: usize,
}

/// Stores all precomputed attack tables.
pub struct AttackTables {
    /// Bishop attack table (~40KB with fancy magics).
    pub bishop_attacks: Vec<Bitboard>,
    /// Rook attack table (~800KB with fancy magics).
    pub rook_attacks: Vec<Bitboard>,
    /// Magic entries for bishops.
    pub bishop_magics: [Magic; 64],
    /// Magic entries for rooks.
    pub rook_magics: [Magic; 64],
}

static ATTACK_TABLES: OnceLock<AttackTables> = OnceLock::new();

/// Gets the global attack tables, initializing if necessary.
pub fn get_attack_tables() -> &'static AttackTables {
    ATTACK_TABLES.get_or_init(AttackTables::new)
}

// Pre-computed magic numbers for bishops (from Chess Programming Wiki / Stockfish).
// These are "fancy" magics that minimize table size.
const BISHOP_MAGICS: [u64; 64] = [
    0x89a1121896040240,
    0x2004844802002010,
    0x2068080051921000,
    0x62880a0220200808,
    0x0004042004000000,
    0x0100822020200011,
    0xc00444222012000a,
    0x0028808801216001,
    0x0400492088408100,
    0x0201c401040c0084,
    0x00840800910a0010,
    0x0000082080240060,
    0x2000840504006000,
    0x30010c4108405004,
    0x1008005410080802,
    0x8144042209100900,
    0x0208081020014400,
    0x004800201208ca00,
    0x0f18140408012008,
    0x1004002802102001,
    0x0841000820080811,
    0x0040200200a42008,
    0x0000800054042000,
    0x88010400410c9000,
    0x0520040470104290,
    0x1004040051500081,
    0x2002081833080021,
    0x000400c00c010142,
    0x941408200c002000,
    0x0658810000806011,
    0x0188071040440a00,
    0x4800404002011c00,
    0x0104442040404200,
    0x0511080200222104,
    0x0004022401120400,
    0x80c0040400080120,
    0x8040010040820802,
    0x0480810700020090,
    0x0102008e00040242,
    0x0809005202050100,
    0x8002024220104080,
    0x0431008804142000,
    0x0019001802081400,
    0x0200014208040080,
    0x3308082008200100,
    0x041010500040c020,
    0x4012020c04210308,
    0x208220a202004080,
    0x0111040120082000,
    0x6803040141280a00,
    0x2101004202410000,
    0x8200000041108022,
    0x0000021082088000,
    0x0002410204010040,
    0x0040100400809000,
    0x0822088220820214,
    0x0040808090012004,
    0x00910224040218c9,
    0x0402814422015008,
    0x0090014004842410,
    0x0001000042304105,
    0x0010008830412a00,
    0x2520081090008908,
    0x40102000a0a60140,
];

// Pre-computed magic numbers for rooks.
const ROOK_MAGICS: [u64; 64] = [
    0x0a8002c000108020,
    0x06c00049b0002001,
    0x0100200010090040,
    0x2480041000800801,
    0x0280028004000800,
    0x0900410008040022,
    0x0280020001001080,
    0x2880002041000080,
    0xa000800080400034,
    0x0004808020004000,
    0x2290802004801000,
    0x0411000d00100020,
    0x0402800800040080,
    0x000b000401004208,
    0x2409000100040200,
    0x0001002100004082,
    0x0022878001e24000,
    0x1090810021004010,
    0x0801030040200012,
    0x0500808008001000,
    0x0a08018014000880,
    0x8000808004000200,
    0x0201008080010200,
    0x0801020000441091,
    0x0000800080204005,
    0x1040200040100048,
    0x0000120200402082,
    0x0d14880480100080,
    0x0012040280080080,
    0x0100040080020080,
    0x9020010080800200,
    0x0813241200148449,
    0x0491604001800080,
    0x0100401000402001,
    0x4820010021001040,
    0x0400402202000812,
    0x0209009005000802,
    0x0810800601800400,
    0x4301083214000150,
    0x204026458e001401,
    0x0040204000808000,
    0x8001008040010020,
    0x8410820820420010,
    0x1003001000090020,
    0x0804040008008080,
    0x0012000810020004,
    0x1000100200040208,
    0x430000a044020001,
    0x0280009023410300,
    0x00e0100040002240,
    0x0000200100401700,
    0x2244100408008080,
    0x0008000400801980,
    0x0002000810040200,
    0x8010100228810400,
    0x2000009044210200,
    0x4080008040102101,
    0x0040002080411d01,
    0x2005524060000901,
    0x0502001008400422,
    0x489a000810200402,
    0x0001004400080a13,
    0x4000011008020084,
    0x0026002114058042,
];

// Bit counts for bishop relevant occupancy (excluding edges).
const BISHOP_BITS: [u8; 64] = [
    6, 5, 5, 5, 5, 5, 5, 6,
    5, 5, 5, 5, 5, 5, 5, 5,
    5, 5, 7, 7, 7, 7, 5, 5,
    5, 5, 7, 9, 9, 7, 5, 5,
    5, 5, 7, 9, 9, 7, 5, 5,
    5, 5, 7, 7, 7, 7, 5, 5,
    5, 5, 5, 5, 5, 5, 5, 5,
    6, 5, 5, 5, 5, 5, 5, 6,
];

// Bit counts for rook relevant occupancy.
const ROOK_BITS: [u8; 64] = [
    12, 11, 11, 11, 11, 11, 11, 12,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    12, 11, 11, 11, 11, 11, 11, 12,
];

impl AttackTables {
    /// Creates and initializes all attack tables.
    pub fn new() -> Self {
        let mut bishop_attacks = Vec::new();
        let mut rook_attacks = Vec::new();
        let mut bishop_magics: [Magic; 64] = std::array::from_fn(|_| Magic {
            mask: Bitboard::EMPTY,
            magic: 0,
            shift: 0,
            offset: 0,
        });
        let mut rook_magics: [Magic; 64] = std::array::from_fn(|_| Magic {
            mask: Bitboard::EMPTY,
            magic: 0,
            shift: 0,
            offset: 0,
        });

        // Initialize bishop tables
        for sq in 0..64 {
            let mask = bishop_mask(sq);
            let bits = BISHOP_BITS[sq as usize];
            let shift = 64 - bits;
            let table_size = 1 << bits;
            let offset = bishop_attacks.len();

            bishop_magics[sq as usize] = Magic {
                mask,
                magic: BISHOP_MAGICS[sq as usize],
                shift,
                offset,
            };

            // Initialize table entries
            bishop_attacks.resize(offset + table_size, Bitboard::EMPTY);

            // Fill attack table for all blocker configurations
            let mut blockers = Bitboard::EMPTY;
            loop {
                let attacks = bishop_attacks_slow(sq, blockers);
                let index = magic_index(&bishop_magics[sq as usize], blockers);
                bishop_attacks[offset + index] = attacks;

                // Carry-Rippler trick to enumerate all subsets
                blockers = Bitboard((blockers.0.wrapping_sub(mask.0)) & mask.0);
                if blockers.is_empty() {
                    break;
                }
            }
        }

        // Initialize rook tables
        for sq in 0..64 {
            let mask = rook_mask(sq);
            let bits = ROOK_BITS[sq as usize];
            let shift = 64 - bits;
            let table_size = 1 << bits;
            let offset = rook_attacks.len();

            rook_magics[sq as usize] = Magic {
                mask,
                magic: ROOK_MAGICS[sq as usize],
                shift,
                offset,
            };

            // Initialize table entries
            rook_attacks.resize(offset + table_size, Bitboard::EMPTY);

            // Fill attack table for all blocker configurations
            let mut blockers = Bitboard::EMPTY;
            loop {
                let attacks = rook_attacks_slow(sq, blockers);
                let index = magic_index(&rook_magics[sq as usize], blockers);
                rook_attacks[offset + index] = attacks;

                blockers = Bitboard((blockers.0.wrapping_sub(mask.0)) & mask.0);
                if blockers.is_empty() {
                    break;
                }
            }
        }

        AttackTables {
            bishop_attacks,
            rook_attacks,
            bishop_magics,
            rook_magics,
        }
    }
}

/// Computes the magic table index for a given blocker configuration.
#[inline]
fn magic_index(magic: &Magic, blockers: Bitboard) -> usize {
    let relevant = blockers & magic.mask;
    ((relevant.0.wrapping_mul(magic.magic)) >> magic.shift) as usize
}

/// Returns bishop attacks for a square given occupied squares.
#[inline]
pub fn bishop_attacks(sq: Square, occupied: Bitboard) -> Bitboard {
    let tables = get_attack_tables();
    let magic = &tables.bishop_magics[sq.index() as usize];
    let index = magic_index(magic, occupied);
    tables.bishop_attacks[magic.offset + index]
}

/// Returns rook attacks for a square given occupied squares.
#[inline]
pub fn rook_attacks(sq: Square, occupied: Bitboard) -> Bitboard {
    let tables = get_attack_tables();
    let magic = &tables.rook_magics[sq.index() as usize];
    let index = magic_index(magic, occupied);
    tables.rook_attacks[magic.offset + index]
}

/// Returns queen attacks (bishop + rook).
#[inline]
pub fn queen_attacks(sq: Square, occupied: Bitboard) -> Bitboard {
    bishop_attacks(sq, occupied) | rook_attacks(sq, occupied)
}

/// Generates the bishop blocker mask for a square (excludes edges).
fn bishop_mask(sq: u8) -> Bitboard {
    let mut mask = 0u64;
    let rank = (sq / 8) as i8;
    let file = (sq % 8) as i8;

    // Diagonal directions, stopping before edges
    for (dr, df) in [(1, 1), (1, -1), (-1, 1), (-1, -1)] {
        let mut r = rank + dr;
        let mut f = file + df;
        while r > 0 && r < 7 && f > 0 && f < 7 {
            mask |= 1u64 << (r * 8 + f);
            r += dr;
            f += df;
        }
    }

    Bitboard(mask)
}

/// Generates the rook blocker mask for a square (excludes edges).
fn rook_mask(sq: u8) -> Bitboard {
    let mut mask = 0u64;
    let rank = sq / 8;
    let file = sq % 8;

    // Horizontal (exclude edge files)
    for f in 1..7 {
        if f != file {
            mask |= 1u64 << (rank * 8 + f);
        }
    }

    // Vertical (exclude edge ranks)
    for r in 1..7 {
        if r != rank {
            mask |= 1u64 << (r * 8 + file);
        }
    }

    Bitboard(mask)
}

/// Slow bishop attack generation (used to build tables).
fn bishop_attacks_slow(sq: u8, blockers: Bitboard) -> Bitboard {
    let mut attacks = 0u64;
    let rank = (sq / 8) as i8;
    let file = (sq % 8) as i8;

    for (dr, df) in [(1, 1), (1, -1), (-1, 1), (-1, -1)] {
        let mut r = rank + dr;
        let mut f = file + df;
        while r >= 0 && r <= 7 && f >= 0 && f <= 7 {
            let bit = 1u64 << (r * 8 + f);
            attacks |= bit;
            if blockers.0 & bit != 0 {
                break;
            }
            r += dr;
            f += df;
        }
    }

    Bitboard(attacks)
}

/// Slow rook attack generation (used to build tables).
fn rook_attacks_slow(sq: u8, blockers: Bitboard) -> Bitboard {
    let mut attacks = 0u64;
    let rank = (sq / 8) as i8;
    let file = (sq % 8) as i8;

    for (dr, df) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
        let mut r = rank + dr;
        let mut f = file + df;
        while r >= 0 && r <= 7 && f >= 0 && f <= 7 {
            let bit = 1u64 << (r * 8 + f);
            attacks |= bit;
            if blockers.0 & bit != 0 {
                break;
            }
            r += dr;
            f += df;
        }
    }

    Bitboard(attacks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chess_core::{File, Rank};

    #[test]
    fn bishop_attacks_empty_board() {
        let sq = Square::new(File::D, Rank::R4);
        let attacks = bishop_attacks(sq, Bitboard::EMPTY);
        // D4 bishop on empty board attacks 13 squares
        assert_eq!(attacks.count(), 13);
    }

    #[test]
    fn rook_attacks_empty_board() {
        let sq = Square::new(File::D, Rank::R4);
        let attacks = rook_attacks(sq, Bitboard::EMPTY);
        // D4 rook on empty board attacks 14 squares
        assert_eq!(attacks.count(), 14);
    }

    #[test]
    fn queen_attacks_empty_board() {
        let sq = Square::new(File::D, Rank::R4);
        let attacks = queen_attacks(sq, Bitboard::EMPTY);
        // D4 queen on empty board attacks 27 squares
        assert_eq!(attacks.count(), 27);
    }

    #[test]
    fn bishop_attacks_with_blockers() {
        let sq = Square::new(File::D, Rank::R4);
        // Place blockers on e5 and c3
        let e5 = Square::new(File::E, Rank::R5);
        let c3 = Square::new(File::C, Rank::R3);
        let blockers = Bitboard::from_square(e5) | Bitboard::from_square(c3);
        let attacks = bishop_attacks(sq, blockers);
        // Should include e5 and c3 (captures) but not beyond
        assert!(attacks.contains(e5));
        assert!(attacks.contains(c3));
        assert!(!attacks.contains(Square::new(File::F, Rank::R6)));
        assert!(!attacks.contains(Square::new(File::B, Rank::R2)));
    }

    #[test]
    fn rook_attacks_with_blockers() {
        let sq = Square::new(File::D, Rank::R4);
        // Place blocker on d6
        let d6 = Square::new(File::D, Rank::R6);
        let blockers = Bitboard::from_square(d6);
        let attacks = rook_attacks(sq, blockers);
        // Should include d6 but not d7 or d8
        assert!(attacks.contains(d6));
        assert!(!attacks.contains(Square::new(File::D, Rank::R7)));
    }

    #[test]
    fn corner_bishop() {
        let attacks = bishop_attacks(Square::A1, Bitboard::EMPTY);
        assert_eq!(attacks.count(), 7); // a1 bishop attacks 7 squares on diagonal
    }

    #[test]
    fn corner_rook() {
        let attacks = rook_attacks(Square::A1, Bitboard::EMPTY);
        assert_eq!(attacks.count(), 14); // a1 rook attacks 14 squares
    }
}
