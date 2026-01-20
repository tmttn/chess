//! Built-in opening book data.
//!
//! This module provides access to the built-in opening database
//! that is compiled into the library.

use crate::database::MoveDatabase;
use crate::opening::{Opening, OpeningMove, OpeningSource};

/// Creates the built-in opening database with common chess openings.
///
/// This database includes popular openings and their main lines,
/// weighted by frequency of play at master level.
#[must_use]
pub fn builtin_database() -> MoveDatabase {
    let mut db = MoveDatabase::new();

    // Starting position - most common first moves
    db.add_position(
        "",
        vec![
            OpeningMove::new("e2e4", 100), // King's Pawn
            OpeningMove::new("d2d4", 90),  // Queen's Pawn
            OpeningMove::new("c2c4", 40),  // English
            OpeningMove::new("g1f3", 30),  // Reti
        ],
    );

    // After 1.e4
    db.add_position(
        "e2e4",
        vec![
            OpeningMove::new("e7e5", 80), // Open Game
            OpeningMove::new("c7c5", 70), // Sicilian
            OpeningMove::new("e7e6", 40), // French
            OpeningMove::new("c7c6", 30), // Caro-Kann
            OpeningMove::new("d7d5", 20), // Scandinavian
        ],
    );

    // After 1.d4
    db.add_position(
        "d2d4",
        vec![
            OpeningMove::new("d7d5", 80), // Closed Game
            OpeningMove::new("g8f6", 70), // Indian Defenses
            OpeningMove::new("e7e6", 30), // Dutch setup
            OpeningMove::new("f7f5", 10), // Dutch Defense
        ],
    );

    // After 1.e4 e5
    db.add_position(
        "e2e4 e7e5",
        vec![
            OpeningMove::new("g1f3", 90), // King's Knight
            OpeningMove::new("f1c4", 30), // Bishop's Opening
            OpeningMove::new("b1c3", 20), // Vienna Game
        ],
    );

    // After 1.e4 e5 2.Nf3
    db.add_position(
        "e2e4 e7e5 g1f3",
        vec![
            OpeningMove::new("b8c6", 90), // Knight's Defense
            OpeningMove::new("g8f6", 40), // Petrov's Defense
            OpeningMove::new("d7d6", 20), // Philidor Defense
        ],
    );

    // After 1.e4 c5 (Sicilian)
    db.add_position(
        "e2e4 c7c5",
        vec![
            OpeningMove::new("g1f3", 80), // Open Sicilian
            OpeningMove::new("b1c3", 40), // Closed Sicilian
            OpeningMove::new("c2c3", 30), // Alapin
        ],
    );

    // After 1.d4 d5
    db.add_position(
        "d2d4 d7d5",
        vec![
            OpeningMove::new("c2c4", 90), // Queen's Gambit
            OpeningMove::new("g1f3", 40), // London/Colle
            OpeningMove::new("c1f4", 30), // London System
        ],
    );

    // After 1.d4 Nf6
    db.add_position(
        "d2d4 g8f6",
        vec![
            OpeningMove::new("c2c4", 90), // Main line
            OpeningMove::new("g1f3", 40), // Slow approach
            OpeningMove::new("c1f4", 30), // London System
        ],
    );

    db
}

/// Creates a collection of built-in chess openings.
///
/// Returns approximately 50 well-known chess openings covering:
/// - Open Games (1.e4 e5)
/// - Semi-Open Games (1.e4, other responses)
/// - Closed Games (1.d4 d5)
/// - Indian Defenses (1.d4 Nf6)
/// - Flank Openings (1.c4, 1.Nf3, etc.)
/// - Gambits
/// - Other popular systems
///
/// All openings include proper ECO codes, UCI move notation, and categorization tags.
#[must_use]
pub fn builtin_openings() -> Vec<Opening> {
    let mut openings = Vec::new();

    // Helper to create tags vector
    let tags = |t: &[&str]| t.iter().map(|s| s.to_string()).collect::<Vec<_>>();

    // ==========================================================================
    // OPEN GAMES (1.e4 e5) - ECO C20-C99
    // ==========================================================================

    // Italian Game - C50
    openings.push(
        Opening::new(
            "italian-game",
            "Italian Game",
            vec!["e2e4", "e7e5", "g1f3", "b8c6", "f1c4"]
                .into_iter()
                .map(String::from)
                .collect(),
            "r1bqkbnr/pppp1ppp/2n5/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R b KQkq - 3 3",
        )
        .with_eco("C50")
        .with_source(OpeningSource::BuiltIn)
        .with_tags(tags(&["open-game", "1.e4", "classical"])),
    );

    // Italian Game: Giuoco Piano - C53
    openings.push(
        Opening::new(
            "giuoco-piano",
            "Giuoco Piano",
            vec!["e2e4", "e7e5", "g1f3", "b8c6", "f1c4", "f8c5"]
                .into_iter()
                .map(String::from)
                .collect(),
            "r1bqk1nr/pppp1ppp/2n5/2b1p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4",
        )
        .with_eco("C53")
        .with_source(OpeningSource::BuiltIn)
        .with_tags(tags(&["open-game", "1.e4", "classical", "italian"])),
    );

    // Ruy Lopez - C60
    openings.push(
        Opening::new(
            "ruy-lopez",
            "Ruy Lopez",
            vec!["e2e4", "e7e5", "g1f3", "b8c6", "f1b5"]
                .into_iter()
                .map(String::from)
                .collect(),
            "r1bqkbnr/pppp1ppp/2n5/1B2p3/4P3/5N2/PPPP1PPP/RNBQK2R b KQkq - 3 3",
        )
        .with_eco("C60")
        .with_source(OpeningSource::BuiltIn)
        .with_tags(tags(&["open-game", "1.e4", "classical"])),
    );

    // Ruy Lopez: Morphy Defense - C65
    openings.push(
        Opening::new(
            "ruy-lopez-morphy-defense",
            "Ruy Lopez: Morphy Defense",
            vec!["e2e4", "e7e5", "g1f3", "b8c6", "f1b5", "a7a6"]
                .into_iter()
                .map(String::from)
                .collect(),
            "r1bqkbnr/1ppp1ppp/p1n5/1B2p3/4P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 0 4",
        )
        .with_eco("C65")
        .with_source(OpeningSource::BuiltIn)
        .with_tags(tags(&["open-game", "1.e4", "classical", "ruy-lopez"])),
    );

    // Scotch Game - C45
    openings.push(
        Opening::new(
            "scotch-game",
            "Scotch Game",
            vec!["e2e4", "e7e5", "g1f3", "b8c6", "d2d4"]
                .into_iter()
                .map(String::from)
                .collect(),
            "r1bqkbnr/pppp1ppp/2n5/4p3/3PP3/5N2/PPP2PPP/RNBQKB1R b KQkq d3 0 3",
        )
        .with_eco("C45")
        .with_source(OpeningSource::BuiltIn)
        .with_tags(tags(&["open-game", "1.e4"])),
    );

    // King's Gambit - C30
    openings.push(
        Opening::new(
            "kings-gambit",
            "King's Gambit",
            vec!["e2e4", "e7e5", "f2f4"]
                .into_iter()
                .map(String::from)
                .collect(),
            "rnbqkbnr/pppp1ppp/8/4p3/4PP2/8/PPPP2PP/RNBQKBNR b KQkq f3 0 2",
        )
        .with_eco("C30")
        .with_source(OpeningSource::BuiltIn)
        .with_tags(tags(&["open-game", "1.e4", "gambit", "aggressive"])),
    );

    // King's Gambit Accepted - C33
    openings.push(
        Opening::new(
            "kings-gambit-accepted",
            "King's Gambit Accepted",
            vec!["e2e4", "e7e5", "f2f4", "e5f4"]
                .into_iter()
                .map(String::from)
                .collect(),
            "rnbqkbnr/pppp1ppp/8/8/4Pp2/8/PPPP2PP/RNBQKBNR w KQkq - 0 3",
        )
        .with_eco("C33")
        .with_source(OpeningSource::BuiltIn)
        .with_tags(tags(&["open-game", "1.e4", "gambit", "aggressive"])),
    );

    // Petrov Defense - C42
    openings.push(
        Opening::new(
            "petrov-defense",
            "Petrov Defense",
            vec!["e2e4", "e7e5", "g1f3", "g8f6"]
                .into_iter()
                .map(String::from)
                .collect(),
            "rnbqkb1r/pppp1ppp/5n2/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 2 3",
        )
        .with_eco("C42")
        .with_source(OpeningSource::BuiltIn)
        .with_tags(tags(&["open-game", "1.e4", "solid"])),
    );

    // Four Knights Game - C47
    openings.push(
        Opening::new(
            "four-knights-game",
            "Four Knights Game",
            vec!["e2e4", "e7e5", "g1f3", "b8c6", "b1c3", "g8f6"]
                .into_iter()
                .map(String::from)
                .collect(),
            "r1bqkb1r/pppp1ppp/2n2n2/4p3/4P3/2N2N2/PPPP1PPP/R1BQKB1R w KQkq - 4 4",
        )
        .with_eco("C47")
        .with_source(OpeningSource::BuiltIn)
        .with_tags(tags(&["open-game", "1.e4", "solid"])),
    );

    // Vienna Game - C25
    openings.push(
        Opening::new(
            "vienna-game",
            "Vienna Game",
            vec!["e2e4", "e7e5", "b1c3"]
                .into_iter()
                .map(String::from)
                .collect(),
            "rnbqkbnr/pppp1ppp/8/4p3/4P3/2N5/PPPP1PPP/R1BQKBNR b KQkq - 1 2",
        )
        .with_eco("C25")
        .with_source(OpeningSource::BuiltIn)
        .with_tags(tags(&["open-game", "1.e4"])),
    );

    // Bishop's Opening - C23
    openings.push(
        Opening::new(
            "bishops-opening",
            "Bishop's Opening",
            vec!["e2e4", "e7e5", "f1c4"]
                .into_iter()
                .map(String::from)
                .collect(),
            "rnbqkbnr/pppp1ppp/8/4p3/2B1P3/8/PPPP1PPP/RNBQK1NR b KQkq - 1 2",
        )
        .with_eco("C23")
        .with_source(OpeningSource::BuiltIn)
        .with_tags(tags(&["open-game", "1.e4"])),
    );

    // Philidor Defense - C41
    openings.push(
        Opening::new(
            "philidor-defense",
            "Philidor Defense",
            vec!["e2e4", "e7e5", "g1f3", "d7d6"]
                .into_iter()
                .map(String::from)
                .collect(),
            "rnbqkbnr/ppp2ppp/3p4/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 0 3",
        )
        .with_eco("C41")
        .with_source(OpeningSource::BuiltIn)
        .with_tags(tags(&["open-game", "1.e4", "solid"])),
    );

    // ==========================================================================
    // SEMI-OPEN GAMES (1.e4 other) - ECO B00-B99, C00-C19
    // ==========================================================================

    // Sicilian Defense - B20
    openings.push(
        Opening::new(
            "sicilian-defense",
            "Sicilian Defense",
            vec!["e2e4", "c7c5"].into_iter().map(String::from).collect(),
            "rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq c6 0 2",
        )
        .with_eco("B20")
        .with_source(OpeningSource::BuiltIn)
        .with_tags(tags(&["semi-open", "1.e4", "sicilian"])),
    );

    // Sicilian: Open Sicilian - B30
    openings.push(
        Opening::new(
            "sicilian-open",
            "Sicilian Defense: Open",
            vec!["e2e4", "c7c5", "g1f3", "b8c6"]
                .into_iter()
                .map(String::from)
                .collect(),
            "r1bqkbnr/pp1ppppp/2n5/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 2 3",
        )
        .with_eco("B30")
        .with_source(OpeningSource::BuiltIn)
        .with_tags(tags(&["semi-open", "1.e4", "sicilian"])),
    );

    // Sicilian Najdorf - B90
    openings.push(
        Opening::new(
            "sicilian-najdorf",
            "Sicilian Defense: Najdorf Variation",
            vec![
                "e2e4", "c7c5", "g1f3", "d7d6", "d2d4", "c5d4", "f3d4", "g8f6", "b1c3", "a7a6",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
            "rnbqkb1r/1p2pppp/p2p1n2/8/3NP3/2N5/PPP2PPP/R1BQKB1R w KQkq - 0 6",
        )
        .with_eco("B90")
        .with_source(OpeningSource::BuiltIn)
        .with_tags(tags(&["semi-open", "1.e4", "sicilian", "aggressive"])),
    );

    // Sicilian Dragon - B70
    openings.push(
        Opening::new(
            "sicilian-dragon",
            "Sicilian Defense: Dragon Variation",
            vec![
                "e2e4", "c7c5", "g1f3", "d7d6", "d2d4", "c5d4", "f3d4", "g8f6", "b1c3", "g7g6",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
            "rnbqkb1r/pp2pp1p/3p1np1/8/3NP3/2N5/PPP2PPP/R1BQKB1R w KQkq - 0 6",
        )
        .with_eco("B70")
        .with_source(OpeningSource::BuiltIn)
        .with_tags(tags(&[
            "semi-open",
            "1.e4",
            "sicilian",
            "aggressive",
            "fianchetto",
        ])),
    );

    // French Defense - C00
    openings.push(
        Opening::new(
            "french-defense",
            "French Defense",
            vec!["e2e4", "e7e6"].into_iter().map(String::from).collect(),
            "rnbqkbnr/pppp1ppp/4p3/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2",
        )
        .with_eco("C00")
        .with_source(OpeningSource::BuiltIn)
        .with_tags(tags(&["semi-open", "1.e4", "solid"])),
    );

    // French Defense: Advance Variation - C02
    openings.push(
        Opening::new(
            "french-advance",
            "French Defense: Advance Variation",
            vec!["e2e4", "e7e6", "d2d4", "d7d5", "e4e5"]
                .into_iter()
                .map(String::from)
                .collect(),
            "rnbqkbnr/ppp2ppp/4p3/3pP3/3P4/8/PPP2PPP/RNBQKBNR b KQkq - 0 3",
        )
        .with_eco("C02")
        .with_source(OpeningSource::BuiltIn)
        .with_tags(tags(&["semi-open", "1.e4", "french"])),
    );

    // Caro-Kann Defense - B10
    openings.push(
        Opening::new(
            "caro-kann-defense",
            "Caro-Kann Defense",
            vec!["e2e4", "c7c6"].into_iter().map(String::from).collect(),
            "rnbqkbnr/pp1ppppp/2p5/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2",
        )
        .with_eco("B10")
        .with_source(OpeningSource::BuiltIn)
        .with_tags(tags(&["semi-open", "1.e4", "solid"])),
    );

    // Caro-Kann: Classical - B18
    openings.push(
        Opening::new(
            "caro-kann-classical",
            "Caro-Kann Defense: Classical Variation",
            vec![
                "e2e4", "c7c6", "d2d4", "d7d5", "b1c3", "d5e4", "c3e4", "c8f5",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
            "rn1qkbnr/pp2pppp/2p5/5b2/3PN3/8/PPP2PPP/R1BQKBNR w KQkq - 1 5",
        )
        .with_eco("B18")
        .with_source(OpeningSource::BuiltIn)
        .with_tags(tags(&["semi-open", "1.e4", "caro-kann", "solid"])),
    );

    // Pirc Defense - B07
    openings.push(
        Opening::new(
            "pirc-defense",
            "Pirc Defense",
            vec!["e2e4", "d7d6", "d2d4", "g8f6", "b1c3", "g7g6"]
                .into_iter()
                .map(String::from)
                .collect(),
            "rnbqkb1r/ppp1pp1p/3p1np1/8/3PP3/2N5/PPP2PPP/R1BQKBNR w KQkq - 0 4",
        )
        .with_eco("B07")
        .with_source(OpeningSource::BuiltIn)
        .with_tags(tags(&["semi-open", "1.e4", "hypermodern", "fianchetto"])),
    );

    // Alekhine Defense - B02
    openings.push(
        Opening::new(
            "alekhine-defense",
            "Alekhine Defense",
            vec!["e2e4", "g8f6"].into_iter().map(String::from).collect(),
            "rnbqkb1r/pppppppp/5n2/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 1 2",
        )
        .with_eco("B02")
        .with_source(OpeningSource::BuiltIn)
        .with_tags(tags(&["semi-open", "1.e4", "hypermodern"])),
    );

    // Scandinavian Defense - B01
    openings.push(
        Opening::new(
            "scandinavian-defense",
            "Scandinavian Defense",
            vec!["e2e4", "d7d5"].into_iter().map(String::from).collect(),
            "rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 2",
        )
        .with_eco("B01")
        .with_source(OpeningSource::BuiltIn)
        .with_tags(tags(&["semi-open", "1.e4"])),
    );

    // Modern Defense - B06
    openings.push(
        Opening::new(
            "modern-defense",
            "Modern Defense",
            vec!["e2e4", "g7g6"].into_iter().map(String::from).collect(),
            "rnbqkbnr/pppppp1p/6p1/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2",
        )
        .with_eco("B06")
        .with_source(OpeningSource::BuiltIn)
        .with_tags(tags(&["semi-open", "1.e4", "hypermodern", "fianchetto"])),
    );

    // ==========================================================================
    // CLOSED GAMES (1.d4 d5) - ECO D00-D69
    // ==========================================================================

    // Queen's Gambit - D06
    openings.push(
        Opening::new(
            "queens-gambit",
            "Queen's Gambit",
            vec!["d2d4", "d7d5", "c2c4"]
                .into_iter()
                .map(String::from)
                .collect(),
            "rnbqkbnr/ppp1pppp/8/3p4/2PP4/8/PP2PPPP/RNBQKBNR b KQkq c3 0 2",
        )
        .with_eco("D06")
        .with_source(OpeningSource::BuiltIn)
        .with_tags(tags(&["closed-game", "1.d4", "gambit"])),
    );

    // Queen's Gambit Declined - D30
    openings.push(
        Opening::new(
            "queens-gambit-declined",
            "Queen's Gambit Declined",
            vec!["d2d4", "d7d5", "c2c4", "e7e6"]
                .into_iter()
                .map(String::from)
                .collect(),
            "rnbqkbnr/ppp2ppp/4p3/3p4/2PP4/8/PP2PPPP/RNBQKBNR w KQkq - 0 3",
        )
        .with_eco("D30")
        .with_source(OpeningSource::BuiltIn)
        .with_tags(tags(&["closed-game", "1.d4", "solid"])),
    );

    // Queen's Gambit Accepted - D20
    openings.push(
        Opening::new(
            "queens-gambit-accepted",
            "Queen's Gambit Accepted",
            vec!["d2d4", "d7d5", "c2c4", "d5c4"]
                .into_iter()
                .map(String::from)
                .collect(),
            "rnbqkbnr/ppp1pppp/8/8/2pP4/8/PP2PPPP/RNBQKBNR w KQkq - 0 3",
        )
        .with_eco("D20")
        .with_source(OpeningSource::BuiltIn)
        .with_tags(tags(&["closed-game", "1.d4"])),
    );

    // Slav Defense - D10
    openings.push(
        Opening::new(
            "slav-defense",
            "Slav Defense",
            vec!["d2d4", "d7d5", "c2c4", "c7c6"]
                .into_iter()
                .map(String::from)
                .collect(),
            "rnbqkbnr/pp2pppp/2p5/3p4/2PP4/8/PP2PPPP/RNBQKBNR w KQkq - 0 3",
        )
        .with_eco("D10")
        .with_source(OpeningSource::BuiltIn)
        .with_tags(tags(&["closed-game", "1.d4", "solid"])),
    );

    // London System - D00
    openings.push(
        Opening::new(
            "london-system",
            "London System",
            vec!["d2d4", "d7d5", "c1f4"]
                .into_iter()
                .map(String::from)
                .collect(),
            "rnbqkbnr/ppp1pppp/8/3p4/3P1B2/8/PPP1PPPP/RN1QKBNR b KQkq - 1 2",
        )
        .with_eco("D00")
        .with_source(OpeningSource::BuiltIn)
        .with_tags(tags(&["closed-game", "1.d4", "system"])),
    );

    // Colle System - D05
    openings.push(
        Opening::new(
            "colle-system",
            "Colle System",
            vec!["d2d4", "d7d5", "g1f3", "g8f6", "e2e3"]
                .into_iter()
                .map(String::from)
                .collect(),
            "rnbqkb1r/ppp1pppp/5n2/3p4/3P4/4PN2/PPP2PPP/RNBQKB1R b KQkq - 0 3",
        )
        .with_eco("D05")
        .with_source(OpeningSource::BuiltIn)
        .with_tags(tags(&["closed-game", "1.d4", "system"])),
    );

    // ==========================================================================
    // INDIAN DEFENSES (1.d4 Nf6) - ECO E00-E99
    // ==========================================================================

    // King's Indian Defense - E60
    openings.push(
        Opening::new(
            "kings-indian-defense",
            "King's Indian Defense",
            vec!["d2d4", "g8f6", "c2c4", "g7g6"]
                .into_iter()
                .map(String::from)
                .collect(),
            "rnbqkb1r/pppppp1p/5np1/8/2PP4/8/PP2PPPP/RNBQKBNR w KQkq - 0 3",
        )
        .with_eco("E60")
        .with_source(OpeningSource::BuiltIn)
        .with_tags(tags(&["indian", "1.d4", "hypermodern", "fianchetto"])),
    );

    // King's Indian: Classical - E90
    openings.push(
        Opening::new(
            "kings-indian-classical",
            "King's Indian Defense: Classical Variation",
            vec![
                "d2d4", "g8f6", "c2c4", "g7g6", "b1c3", "f8g7", "e2e4", "d7d6", "g1f3",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
            "rnbqk2r/ppp1ppbp/3p1np1/8/2PPP3/2N2N2/PP3PPP/R1BQKB1R b KQkq - 1 5",
        )
        .with_eco("E90")
        .with_source(OpeningSource::BuiltIn)
        .with_tags(tags(&["indian", "1.d4", "kings-indian", "aggressive"])),
    );

    // Nimzo-Indian Defense - E20
    openings.push(
        Opening::new(
            "nimzo-indian-defense",
            "Nimzo-Indian Defense",
            vec!["d2d4", "g8f6", "c2c4", "e7e6", "b1c3", "f8b4"]
                .into_iter()
                .map(String::from)
                .collect(),
            "rnbqk2r/pppp1ppp/4pn2/8/1bPP4/2N5/PP2PPPP/R1BQKBNR w KQkq - 2 4",
        )
        .with_eco("E20")
        .with_source(OpeningSource::BuiltIn)
        .with_tags(tags(&["indian", "1.d4", "positional"])),
    );

    // Queen's Indian Defense - E12
    openings.push(
        Opening::new(
            "queens-indian-defense",
            "Queen's Indian Defense",
            vec!["d2d4", "g8f6", "c2c4", "e7e6", "g1f3", "b7b6"]
                .into_iter()
                .map(String::from)
                .collect(),
            "rnbqkb1r/p1pp1ppp/1p2pn2/8/2PP4/5N2/PP2PPPP/RNBQKB1R w KQkq - 0 4",
        )
        .with_eco("E12")
        .with_source(OpeningSource::BuiltIn)
        .with_tags(tags(&["indian", "1.d4", "solid", "fianchetto"])),
    );

    // Grunfeld Defense - D80
    openings.push(
        Opening::new(
            "grunfeld-defense",
            "Grunfeld Defense",
            vec!["d2d4", "g8f6", "c2c4", "g7g6", "b1c3", "d7d5"]
                .into_iter()
                .map(String::from)
                .collect(),
            "rnbqkb1r/ppp1pp1p/5np1/3p4/2PP4/2N5/PP2PPPP/R1BQKBNR w KQkq d6 0 4",
        )
        .with_eco("D80")
        .with_source(OpeningSource::BuiltIn)
        .with_tags(tags(&["indian", "1.d4", "hypermodern", "fianchetto"])),
    );

    // Bogo-Indian Defense - E11
    openings.push(
        Opening::new(
            "bogo-indian-defense",
            "Bogo-Indian Defense",
            vec!["d2d4", "g8f6", "c2c4", "e7e6", "g1f3", "f8b4"]
                .into_iter()
                .map(String::from)
                .collect(),
            "rnbqk2r/pppp1ppp/4pn2/8/1bPP4/5N2/PP2PPPP/RNBQKB1R w KQkq - 2 4",
        )
        .with_eco("E11")
        .with_source(OpeningSource::BuiltIn)
        .with_tags(tags(&["indian", "1.d4", "solid"])),
    );

    // Catalan Opening - E00
    openings.push(
        Opening::new(
            "catalan-opening",
            "Catalan Opening",
            vec!["d2d4", "g8f6", "c2c4", "e7e6", "g2g3"]
                .into_iter()
                .map(String::from)
                .collect(),
            "rnbqkb1r/pppp1ppp/4pn2/8/2PP4/6P1/PP2PP1P/RNBQKBNR b KQkq - 0 3",
        )
        .with_eco("E00")
        .with_source(OpeningSource::BuiltIn)
        .with_tags(tags(&["indian", "1.d4", "fianchetto", "positional"])),
    );

    // ==========================================================================
    // FLANK OPENINGS - ECO A00-A39
    // ==========================================================================

    // English Opening - A10
    openings.push(
        Opening::new(
            "english-opening",
            "English Opening",
            vec!["c2c4"].into_iter().map(String::from).collect(),
            "rnbqkbnr/pppppppp/8/8/2P5/8/PP1PPPPP/RNBQKBNR b KQkq c3 0 1",
        )
        .with_eco("A10")
        .with_source(OpeningSource::BuiltIn)
        .with_tags(tags(&["flank", "hypermodern"])),
    );

    // English: Symmetrical - A30
    openings.push(
        Opening::new(
            "english-symmetrical",
            "English Opening: Symmetrical Variation",
            vec!["c2c4", "c7c5"].into_iter().map(String::from).collect(),
            "rnbqkbnr/pp1ppppp/8/2p5/2P5/8/PP1PPPPP/RNBQKBNR w KQkq c6 0 2",
        )
        .with_eco("A30")
        .with_source(OpeningSource::BuiltIn)
        .with_tags(tags(&["flank", "english", "hypermodern"])),
    );

    // Reti Opening - A04
    openings.push(
        Opening::new(
            "reti-opening",
            "Reti Opening",
            vec!["g1f3"].into_iter().map(String::from).collect(),
            "rnbqkbnr/pppppppp/8/8/8/5N2/PPPPPPPP/RNBQKB1R b KQkq - 1 1",
        )
        .with_eco("A04")
        .with_source(OpeningSource::BuiltIn)
        .with_tags(tags(&["flank", "hypermodern"])),
    );

    // Reti: King's Indian Attack setup - A07
    openings.push(
        Opening::new(
            "reti-kings-indian-attack",
            "Reti Opening: King's Indian Attack",
            vec!["g1f3", "d7d5", "g2g3"]
                .into_iter()
                .map(String::from)
                .collect(),
            "rnbqkbnr/ppp1pppp/8/3p4/8/5NP1/PPPPPP1P/RNBQKB1R b KQkq - 0 2",
        )
        .with_eco("A07")
        .with_source(OpeningSource::BuiltIn)
        .with_tags(tags(&["flank", "hypermodern", "fianchetto"])),
    );

    // Bird's Opening - A02
    openings.push(
        Opening::new(
            "birds-opening",
            "Bird's Opening",
            vec!["f2f4"].into_iter().map(String::from).collect(),
            "rnbqkbnr/pppppppp/8/8/5P2/8/PPPPP1PP/RNBQKBNR b KQkq f3 0 1",
        )
        .with_eco("A02")
        .with_source(OpeningSource::BuiltIn)
        .with_tags(tags(&["flank", "aggressive"])),
    );

    // Larsen's Opening - A01
    openings.push(
        Opening::new(
            "larsens-opening",
            "Larsen's Opening",
            vec!["b2b3"].into_iter().map(String::from).collect(),
            "rnbqkbnr/pppppppp/8/8/8/1P6/P1PPPPPP/RNBQKBNR b KQkq - 0 1",
        )
        .with_eco("A01")
        .with_source(OpeningSource::BuiltIn)
        .with_tags(tags(&["flank", "hypermodern", "fianchetto"])),
    );

    // King's Indian Attack - A07
    openings.push(
        Opening::new(
            "kings-indian-attack",
            "King's Indian Attack",
            vec!["g1f3", "d7d5", "g2g3", "g8f6", "f1g2"]
                .into_iter()
                .map(String::from)
                .collect(),
            "rnbqkb1r/ppp1pppp/5n2/3p4/8/5NP1/PPPPPPBP/RNBQK2R b KQkq - 2 3",
        )
        .with_eco("A07")
        .with_source(OpeningSource::BuiltIn)
        .with_tags(tags(&["flank", "system", "fianchetto"])),
    );

    // ==========================================================================
    // GAMBITS
    // ==========================================================================

    // Evans Gambit - C51
    openings.push(
        Opening::new(
            "evans-gambit",
            "Evans Gambit",
            vec!["e2e4", "e7e5", "g1f3", "b8c6", "f1c4", "f8c5", "b2b4"]
                .into_iter()
                .map(String::from)
                .collect(),
            "r1bqk1nr/pppp1ppp/2n5/2b1p3/1PB1P3/5N2/P1PP1PPP/RNBQK2R b KQkq b3 0 4",
        )
        .with_eco("C51")
        .with_source(OpeningSource::BuiltIn)
        .with_tags(tags(&[
            "open-game",
            "1.e4",
            "gambit",
            "aggressive",
            "italian",
        ])),
    );

    // Smith-Morra Gambit - B21
    openings.push(
        Opening::new(
            "smith-morra-gambit",
            "Sicilian Defense: Smith-Morra Gambit",
            vec!["e2e4", "c7c5", "d2d4", "c5d4", "c2c3"]
                .into_iter()
                .map(String::from)
                .collect(),
            "rnbqkbnr/pp1ppppp/8/8/3pP3/2P5/PP3PPP/RNBQKBNR b KQkq - 0 3",
        )
        .with_eco("B21")
        .with_source(OpeningSource::BuiltIn)
        .with_tags(tags(&[
            "semi-open",
            "1.e4",
            "gambit",
            "aggressive",
            "sicilian",
        ])),
    );

    // Budapest Gambit - A51
    openings.push(
        Opening::new(
            "budapest-gambit",
            "Budapest Gambit",
            vec!["d2d4", "g8f6", "c2c4", "e7e5"]
                .into_iter()
                .map(String::from)
                .collect(),
            "rnbqkb1r/pppp1ppp/5n2/4p3/2PP4/8/PP2PPPP/RNBQKBNR w KQkq e6 0 3",
        )
        .with_eco("A51")
        .with_source(OpeningSource::BuiltIn)
        .with_tags(tags(&["indian", "1.d4", "gambit", "aggressive"])),
    );

    // Benko Gambit - A57
    openings.push(
        Opening::new(
            "benko-gambit",
            "Benko Gambit",
            vec!["d2d4", "g8f6", "c2c4", "c7c5", "d4d5", "b7b5"]
                .into_iter()
                .map(String::from)
                .collect(),
            "rnbqkb1r/p2ppppp/5n2/1ppP4/2P5/8/PP2PPPP/RNBQKBNR w KQkq b6 0 4",
        )
        .with_eco("A57")
        .with_source(OpeningSource::BuiltIn)
        .with_tags(tags(&["indian", "1.d4", "gambit"])),
    );

    // Danish Gambit - C21
    openings.push(
        Opening::new(
            "danish-gambit",
            "Danish Gambit",
            vec!["e2e4", "e7e5", "d2d4", "e5d4", "c2c3"]
                .into_iter()
                .map(String::from)
                .collect(),
            "rnbqkbnr/pppp1ppp/8/8/3pP3/2P5/PP3PPP/RNBQKBNR b KQkq - 0 3",
        )
        .with_eco("C21")
        .with_source(OpeningSource::BuiltIn)
        .with_tags(tags(&["open-game", "1.e4", "gambit", "aggressive"])),
    );

    // Blackmar-Diemer Gambit - D00
    openings.push(
        Opening::new(
            "blackmar-diemer-gambit",
            "Blackmar-Diemer Gambit",
            vec!["d2d4", "d7d5", "e2e4", "d5e4", "b1c3"]
                .into_iter()
                .map(String::from)
                .collect(),
            "rnbqkbnr/ppp1pppp/8/8/3Pp3/2N5/PPP2PPP/R1BQKBNR b KQkq - 1 3",
        )
        .with_eco("D00")
        .with_source(OpeningSource::BuiltIn)
        .with_tags(tags(&["closed-game", "1.d4", "gambit", "aggressive"])),
    );

    // ==========================================================================
    // OTHER OPENINGS
    // ==========================================================================

    // Dutch Defense - A80
    openings.push(
        Opening::new(
            "dutch-defense",
            "Dutch Defense",
            vec!["d2d4", "f7f5"].into_iter().map(String::from).collect(),
            "rnbqkbnr/ppppp1pp/8/5p2/3P4/8/PPP1PPPP/RNBQKBNR w KQkq f6 0 2",
        )
        .with_eco("A80")
        .with_source(OpeningSource::BuiltIn)
        .with_tags(tags(&["closed-game", "1.d4", "aggressive"])),
    );

    // Dutch: Leningrad - A87
    openings.push(
        Opening::new(
            "dutch-leningrad",
            "Dutch Defense: Leningrad Variation",
            vec!["d2d4", "f7f5", "g2g3", "g8f6", "f1g2", "g7g6"]
                .into_iter()
                .map(String::from)
                .collect(),
            "rnbqkb1r/ppppp2p/5np1/5p2/3P4/6P1/PPP1PPBP/RNBQK1NR w KQkq - 0 4",
        )
        .with_eco("A87")
        .with_source(OpeningSource::BuiltIn)
        .with_tags(tags(&["closed-game", "1.d4", "dutch", "fianchetto"])),
    );

    // Benoni Defense - A60
    openings.push(
        Opening::new(
            "benoni-defense",
            "Benoni Defense",
            vec!["d2d4", "g8f6", "c2c4", "c7c5", "d4d5"]
                .into_iter()
                .map(String::from)
                .collect(),
            "rnbqkb1r/pp1ppppp/5n2/2pP4/2P5/8/PP2PPPP/RNBQKBNR b KQkq - 0 3",
        )
        .with_eco("A60")
        .with_source(OpeningSource::BuiltIn)
        .with_tags(tags(&["indian", "1.d4", "aggressive"])),
    );

    // Modern Benoni - A70
    openings.push(
        Opening::new(
            "modern-benoni",
            "Modern Benoni",
            vec![
                "d2d4", "g8f6", "c2c4", "c7c5", "d4d5", "e7e6", "b1c3", "e6d5", "c4d5", "d7d6",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
            "rnbqkb1r/pp3ppp/3p1n2/2pP4/8/2N5/PP2PPPP/R1BQKBNR w KQkq - 0 6",
        )
        .with_eco("A70")
        .with_source(OpeningSource::BuiltIn)
        .with_tags(tags(&["indian", "1.d4", "benoni", "aggressive"])),
    );

    // Trompowsky Attack - A45
    openings.push(
        Opening::new(
            "trompowsky-attack",
            "Trompowsky Attack",
            vec!["d2d4", "g8f6", "c1g5"]
                .into_iter()
                .map(String::from)
                .collect(),
            "rnbqkb1r/pppppppp/5n2/6B1/3P4/8/PPP1PPPP/RN1QKBNR b KQkq - 2 2",
        )
        .with_eco("A45")
        .with_source(OpeningSource::BuiltIn)
        .with_tags(tags(&["closed-game", "1.d4", "system"])),
    );

    // Torre Attack - A46
    openings.push(
        Opening::new(
            "torre-attack",
            "Torre Attack",
            vec!["d2d4", "g8f6", "g1f3", "e7e6", "c1g5"]
                .into_iter()
                .map(String::from)
                .collect(),
            "rnbqkb1r/pppp1ppp/4pn2/6B1/3P4/5N2/PPP1PPPP/RN1QKB1R b KQkq - 2 3",
        )
        .with_eco("A46")
        .with_source(OpeningSource::BuiltIn)
        .with_tags(tags(&["closed-game", "1.d4", "system"])),
    );

    openings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_database_not_empty() {
        let db = builtin_database();
        assert!(!db.is_empty());
    }

    #[test]
    fn test_starting_position_has_moves() {
        let db = builtin_database();
        let moves = db.lookup("").unwrap();
        assert!(!moves.is_empty());
        // e2e4 should be the most common
        assert_eq!(moves[0].uci, "e2e4");
    }

    #[test]
    fn test_e4_response() {
        let db = builtin_database();
        let moves = db.lookup("e2e4").unwrap();
        assert!(!moves.is_empty());
        // e7e5 should be available as a response
        assert!(moves.iter().any(|m| m.uci == "e7e5"));
    }

    #[test]
    fn test_sicilian_continuation() {
        let db = builtin_database();
        let moves = db.lookup("e2e4 c7c5").unwrap();
        assert!(!moves.is_empty());
        // Nf3 should be the main response
        assert!(moves.iter().any(|m| m.uci == "g1f3"));
    }

    // =========================================================================
    // Tests for builtin_openings()
    // =========================================================================

    #[test]
    fn test_builtin_openings_not_empty() {
        let openings = builtin_openings();
        assert!(!openings.is_empty());
        // Should have approximately 50 openings
        assert!(
            openings.len() >= 45,
            "Expected at least 45 openings, got {}",
            openings.len()
        );
    }

    #[test]
    fn test_builtin_openings_unique_ids() {
        let openings = builtin_openings();
        let mut ids: Vec<&str> = openings.iter().map(|o| o.id.as_str()).collect();
        let original_count = ids.len();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), original_count, "All opening IDs must be unique");
    }

    #[test]
    fn test_builtin_openings_all_have_eco() {
        let openings = builtin_openings();
        for opening in &openings {
            assert!(
                opening.eco.is_some(),
                "Opening '{}' is missing ECO code",
                opening.name
            );
            let eco = opening.eco.as_ref().unwrap();
            // ECO codes are A00-E99
            assert!(
                eco.len() >= 2 && eco.len() <= 3,
                "Opening '{}' has invalid ECO code length: {}",
                opening.name,
                eco
            );
            let first_char = eco.chars().next().unwrap();
            assert!(
                ('A'..='E').contains(&first_char),
                "Opening '{}' has invalid ECO code prefix: {}",
                opening.name,
                eco
            );
        }
    }

    #[test]
    fn test_builtin_openings_all_have_valid_moves() {
        let openings = builtin_openings();
        for opening in &openings {
            assert!(
                !opening.moves.is_empty(),
                "Opening '{}' has no moves",
                opening.name
            );
            for (i, mv) in opening.moves.iter().enumerate() {
                // UCI moves are 4 characters (e.g., e2e4) or 5 for promotions (e.g., e7e8q)
                assert!(
                    mv.len() >= 4 && mv.len() <= 5,
                    "Opening '{}' has invalid move at position {}: '{}'",
                    opening.name,
                    i,
                    mv
                );
                // First two characters should be a valid square
                let from_file = mv.chars().next().unwrap();
                let from_rank = mv.chars().nth(1).unwrap();
                assert!(
                    ('a'..='h').contains(&from_file) && ('1'..='8').contains(&from_rank),
                    "Opening '{}' has invalid source square in move '{}' at position {}",
                    opening.name,
                    mv,
                    i
                );
                // Characters 3-4 should be a valid square
                let to_file = mv.chars().nth(2).unwrap();
                let to_rank = mv.chars().nth(3).unwrap();
                assert!(
                    ('a'..='h').contains(&to_file) && ('1'..='8').contains(&to_rank),
                    "Opening '{}' has invalid target square in move '{}' at position {}",
                    opening.name,
                    mv,
                    i
                );
            }
        }
    }

    #[test]
    fn test_builtin_openings_all_have_builtin_source() {
        let openings = builtin_openings();
        for opening in &openings {
            assert_eq!(
                opening.source,
                OpeningSource::BuiltIn,
                "Opening '{}' should have BuiltIn source",
                opening.name
            );
        }
    }

    #[test]
    fn test_builtin_openings_category_coverage() {
        let openings = builtin_openings();

        // Count openings by category tags
        let open_games = openings.iter().filter(|o| o.has_tag("open-game")).count();
        let semi_open = openings.iter().filter(|o| o.has_tag("semi-open")).count();
        let closed_games = openings.iter().filter(|o| o.has_tag("closed-game")).count();
        let indian = openings.iter().filter(|o| o.has_tag("indian")).count();
        let flank = openings.iter().filter(|o| o.has_tag("flank")).count();
        let gambits = openings.iter().filter(|o| o.has_tag("gambit")).count();

        // Each category should have at least some representation
        assert!(
            open_games >= 5,
            "Expected at least 5 open games, got {}",
            open_games
        );
        assert!(
            semi_open >= 5,
            "Expected at least 5 semi-open games, got {}",
            semi_open
        );
        assert!(
            closed_games >= 3,
            "Expected at least 3 closed games, got {}",
            closed_games
        );
        assert!(
            indian >= 5,
            "Expected at least 5 Indian defenses, got {}",
            indian
        );
        assert!(
            flank >= 3,
            "Expected at least 3 flank openings, got {}",
            flank
        );
        assert!(gambits >= 5, "Expected at least 5 gambits, got {}", gambits);
    }

    #[test]
    fn test_builtin_openings_id_format() {
        let openings = builtin_openings();
        for opening in &openings {
            // IDs should be kebab-case (lowercase with hyphens)
            assert!(
                opening
                    .id
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c == '-'),
                "Opening ID '{}' should be kebab-case",
                opening.id
            );
            // No double hyphens
            assert!(
                !opening.id.contains("--"),
                "Opening ID '{}' should not contain double hyphens",
                opening.id
            );
            // Should not start or end with hyphen
            assert!(
                !opening.id.starts_with('-') && !opening.id.ends_with('-'),
                "Opening ID '{}' should not start or end with hyphen",
                opening.id
            );
        }
    }

    #[test]
    fn test_builtin_openings_specific_openings_exist() {
        let openings = builtin_openings();
        let ids: Vec<&str> = openings.iter().map(|o| o.id.as_str()).collect();

        // Check for some key openings
        let expected = [
            "italian-game",
            "ruy-lopez",
            "sicilian-defense",
            "french-defense",
            "caro-kann-defense",
            "queens-gambit",
            "kings-indian-defense",
            "nimzo-indian-defense",
            "english-opening",
            "london-system",
        ];

        for name in expected {
            assert!(ids.contains(&name), "Expected opening '{}' not found", name);
        }
    }
}
