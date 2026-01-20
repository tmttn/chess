//! Core opening types and structures.

use serde::{Deserialize, Serialize};

/// The source of an opening definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum OpeningSource {
    /// Built-in opening from the crate's default database.
    #[default]
    BuiltIn,
    /// Opening from an ECO (Encyclopedia of Chess Openings) database.
    Eco,
    /// Opening from Lichess opening database.
    Lichess,
    /// Custom user-defined opening.
    Custom,
}

impl std::fmt::Display for OpeningSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpeningSource::BuiltIn => write!(f, "builtin"),
            OpeningSource::Eco => write!(f, "eco"),
            OpeningSource::Lichess => write!(f, "lichess"),
            OpeningSource::Custom => write!(f, "custom"),
        }
    }
}

/// Statistics about an opening's performance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpeningStats {
    /// Total number of games played with this opening.
    pub games_played: u64,
    /// Win rate for white (0.0 to 1.0).
    pub white_wins: f32,
    /// Draw rate (0.0 to 1.0).
    pub draws: f32,
    /// Win rate for black (0.0 to 1.0).
    pub black_wins: f32,
}

impl OpeningStats {
    /// Creates new opening statistics.
    ///
    /// # Arguments
    ///
    /// * `games_played` - Total number of games
    /// * `white_wins` - White win rate (0.0 to 1.0)
    /// * `draws` - Draw rate (0.0 to 1.0)
    /// * `black_wins` - Black win rate (0.0 to 1.0)
    #[must_use]
    pub fn new(games_played: u64, white_wins: f32, draws: f32, black_wins: f32) -> Self {
        Self {
            games_played,
            white_wins,
            draws,
            black_wins,
        }
    }

    /// Validates that the win rates sum to approximately 1.0.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        let sum = self.white_wins + self.draws + self.black_wins;
        (sum - 1.0).abs() < 0.01
    }
}

impl Default for OpeningStats {
    fn default() -> Self {
        Self {
            games_played: 0,
            white_wins: 0.0,
            draws: 0.0,
            black_wins: 0.0,
        }
    }
}

/// Represents a chess opening with its name, moves, and metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Opening {
    /// Unique identifier for this opening (e.g., "italian-game").
    pub id: String,
    /// Human-readable name of the opening (e.g., "Italian Game").
    pub name: String,
    /// ECO code for this opening (e.g., "C50"), if known.
    pub eco: Option<String>,
    /// The sequence of moves in UCI notation.
    pub moves: Vec<String>,
    /// FEN string representing the position after all moves.
    pub fen: String,
    /// Where this opening definition came from.
    pub source: OpeningSource,
    /// Tags for categorizing the opening (e.g., ["open", "1.e4"]).
    pub tags: Vec<String>,
    /// Performance statistics for this opening, if available.
    pub stats: Option<OpeningStats>,
}

/// The standard starting position FEN.
pub const STARTING_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

impl Opening {
    /// Creates a new opening with required fields.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier (e.g., "italian-game")
    /// * `name` - Human-readable name (e.g., "Italian Game")
    /// * `moves` - Sequence of moves in UCI notation
    /// * `fen` - FEN string for the position after all moves
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        moves: Vec<String>,
        fen: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            eco: None,
            moves,
            fen: fen.into(),
            source: OpeningSource::default(),
            tags: Vec::new(),
            stats: None,
        }
    }

    /// Sets the ECO code for this opening.
    #[must_use]
    pub fn with_eco(mut self, eco: impl Into<String>) -> Self {
        self.eco = Some(eco.into());
        self
    }

    /// Sets the source of this opening.
    #[must_use]
    pub fn with_source(mut self, source: OpeningSource) -> Self {
        self.source = source;
        self
    }

    /// Sets the tags for this opening.
    #[must_use]
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Sets the statistics for this opening.
    #[must_use]
    pub fn with_stats(mut self, stats: OpeningStats) -> Self {
        self.stats = Some(stats);
        self
    }

    /// Returns the number of half-moves (plies) in this opening.
    #[must_use]
    pub fn ply_count(&self) -> usize {
        self.moves.len()
    }

    /// Returns the number of full moves in this opening.
    ///
    /// A full move consists of one move by white and one by black.
    #[must_use]
    pub fn move_count(&self) -> usize {
        self.moves.len().div_ceil(2)
    }

    /// Returns true if this opening has an ECO code.
    #[must_use]
    pub fn has_eco(&self) -> bool {
        self.eco.is_some()
    }

    /// Returns true if this opening has performance statistics.
    #[must_use]
    pub fn has_stats(&self) -> bool {
        self.stats.is_some()
    }

    /// Returns true if this opening has the specified tag.
    #[must_use]
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }
}

/// A single move from an opening book with associated metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpeningMove {
    /// The move in UCI notation (e.g., "e2e4").
    pub uci: String,
    /// Weight/frequency of this move (higher = more common).
    pub weight: u32,
}

impl OpeningMove {
    /// Creates a new opening move with the given UCI notation and weight.
    #[must_use]
    pub fn new(uci: impl Into<String>, weight: u32) -> Self {
        Self {
            uci: uci.into(),
            weight,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opening_source_display() {
        assert_eq!(OpeningSource::BuiltIn.to_string(), "builtin");
        assert_eq!(OpeningSource::Eco.to_string(), "eco");
        assert_eq!(OpeningSource::Lichess.to_string(), "lichess");
        assert_eq!(OpeningSource::Custom.to_string(), "custom");
    }

    #[test]
    fn test_opening_source_default() {
        assert_eq!(OpeningSource::default(), OpeningSource::BuiltIn);
    }

    #[test]
    fn test_opening_stats_new() {
        let stats = OpeningStats::new(1000, 0.40, 0.35, 0.25);
        assert_eq!(stats.games_played, 1000);
        assert!((stats.white_wins - 0.40).abs() < f32::EPSILON);
        assert!((stats.draws - 0.35).abs() < f32::EPSILON);
        assert!((stats.black_wins - 0.25).abs() < f32::EPSILON);
    }

    #[test]
    fn test_opening_stats_is_valid() {
        let valid_stats = OpeningStats::new(1000, 0.40, 0.35, 0.25);
        assert!(valid_stats.is_valid());

        let invalid_stats = OpeningStats::new(1000, 0.50, 0.50, 0.50);
        assert!(!invalid_stats.is_valid());
    }

    #[test]
    fn test_opening_stats_default() {
        let stats = OpeningStats::default();
        assert_eq!(stats.games_played, 0);
        assert!((stats.white_wins).abs() < f32::EPSILON);
        assert!((stats.draws).abs() < f32::EPSILON);
        assert!((stats.black_wins).abs() < f32::EPSILON);
    }

    #[test]
    fn test_opening_new() {
        let opening = Opening::new(
            "italian-game",
            "Italian Game",
            vec![
                "e2e4".to_string(),
                "e7e5".to_string(),
                "g1f3".to_string(),
                "b8c6".to_string(),
                "f1c4".to_string(),
            ],
            "r1bqkbnr/pppp1ppp/2n5/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R b KQkq - 3 3",
        );
        assert_eq!(opening.id, "italian-game");
        assert_eq!(opening.name, "Italian Game");
        assert!(opening.eco.is_none());
        assert_eq!(opening.moves.len(), 5);
        assert_eq!(opening.source, OpeningSource::BuiltIn);
        assert!(opening.tags.is_empty());
        assert!(opening.stats.is_none());
    }

    #[test]
    fn test_opening_with_eco() {
        let opening = Opening::new(
            "sicilian-defense",
            "Sicilian Defense",
            vec!["e2e4".to_string(), "c7c5".to_string()],
            "rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq c6 0 2",
        )
        .with_eco("B20");

        assert_eq!(opening.eco, Some("B20".to_string()));
        assert!(opening.has_eco());
    }

    #[test]
    fn test_opening_with_source() {
        let opening = Opening::new(
            "queens-gambit",
            "Queen's Gambit",
            vec!["d2d4".to_string(), "d7d5".to_string(), "c2c4".to_string()],
            "rnbqkbnr/ppp1pppp/8/3p4/2PP4/8/PP2PPPP/RNBQKBNR b KQkq c3 0 2",
        )
        .with_source(OpeningSource::Lichess);

        assert_eq!(opening.source, OpeningSource::Lichess);
    }

    #[test]
    fn test_opening_with_tags() {
        let opening = Opening::new(
            "kings-pawn",
            "King's Pawn Game",
            vec!["e2e4".to_string()],
            "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1",
        )
        .with_tags(vec!["open".to_string(), "1.e4".to_string()]);

        assert_eq!(opening.tags.len(), 2);
        assert!(opening.has_tag("open"));
        assert!(opening.has_tag("1.e4"));
        assert!(!opening.has_tag("closed"));
    }

    #[test]
    fn test_opening_with_stats() {
        let stats = OpeningStats::new(50000, 0.38, 0.32, 0.30);
        let opening = Opening::new(
            "ruy-lopez",
            "Ruy Lopez",
            vec![
                "e2e4".to_string(),
                "e7e5".to_string(),
                "g1f3".to_string(),
                "b8c6".to_string(),
                "f1b5".to_string(),
            ],
            "r1bqkbnr/pppp1ppp/2n5/1B2p3/4P3/5N2/PPPP1PPP/RNBQK2R b KQkq - 3 3",
        )
        .with_stats(stats);

        assert!(opening.has_stats());
        let opening_stats = opening.stats.as_ref().unwrap();
        assert_eq!(opening_stats.games_played, 50000);
    }

    #[test]
    fn test_opening_builder_chain() {
        let opening = Opening::new(
            "french-defense",
            "French Defense",
            vec!["e2e4".to_string(), "e7e6".to_string()],
            "rnbqkbnr/pppp1ppp/4p3/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2",
        )
        .with_eco("C00")
        .with_source(OpeningSource::Eco)
        .with_tags(vec!["closed".to_string(), "1.e4".to_string()])
        .with_stats(OpeningStats::new(100000, 0.37, 0.33, 0.30));

        assert_eq!(opening.id, "french-defense");
        assert_eq!(opening.eco, Some("C00".to_string()));
        assert_eq!(opening.source, OpeningSource::Eco);
        assert_eq!(opening.tags.len(), 2);
        assert!(opening.stats.is_some());
    }

    #[test]
    fn test_opening_ply_count() {
        let opening = Opening::new(
            "test",
            "Test",
            vec![
                "e2e4".to_string(),
                "e7e5".to_string(),
                "g1f3".to_string(),
                "b8c6".to_string(),
                "f1b5".to_string(),
            ],
            "test-fen",
        );
        assert_eq!(opening.ply_count(), 5);
    }

    #[test]
    fn test_opening_move_count() {
        // 5 plies = 3 full moves (1.e4 e5 2.Nf3 Nc6 3.Bb5)
        let opening_5_ply = Opening::new(
            "test",
            "Test",
            vec![
                "e2e4".to_string(),
                "e7e5".to_string(),
                "g1f3".to_string(),
                "b8c6".to_string(),
                "f1b5".to_string(),
            ],
            "test-fen",
        );
        assert_eq!(opening_5_ply.move_count(), 3);

        // 4 plies = 2 full moves (1.e4 e5 2.Nf3 Nc6)
        let opening_4_ply = Opening::new(
            "test",
            "Test",
            vec![
                "e2e4".to_string(),
                "e7e5".to_string(),
                "g1f3".to_string(),
                "b8c6".to_string(),
            ],
            "test-fen",
        );
        assert_eq!(opening_4_ply.move_count(), 2);

        // Empty opening
        let empty_opening = Opening::new("test", "Test", vec![], "test-fen");
        assert_eq!(empty_opening.move_count(), 0);
    }

    #[test]
    fn test_opening_move_new() {
        let mv = OpeningMove::new("e2e4", 100);
        assert_eq!(mv.uci, "e2e4");
        assert_eq!(mv.weight, 100);
    }

    #[test]
    fn test_opening_serde_roundtrip() {
        let opening = Opening::new(
            "caro-kann",
            "Caro-Kann Defense",
            vec!["e2e4".to_string(), "c7c6".to_string()],
            "rnbqkbnr/pp1ppppp/2p5/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2",
        )
        .with_eco("B10")
        .with_source(OpeningSource::Eco)
        .with_tags(vec!["solid".to_string()])
        .with_stats(OpeningStats::new(80000, 0.36, 0.34, 0.30));

        let json = serde_json::to_string(&opening).expect("serialization failed");
        let deserialized: Opening = serde_json::from_str(&json).expect("deserialization failed");

        assert_eq!(opening.id, deserialized.id);
        assert_eq!(opening.name, deserialized.name);
        assert_eq!(opening.eco, deserialized.eco);
        assert_eq!(opening.moves, deserialized.moves);
        assert_eq!(opening.fen, deserialized.fen);
        assert_eq!(opening.source, deserialized.source);
        assert_eq!(opening.tags, deserialized.tags);

        let original_stats = opening.stats.as_ref().unwrap();
        let deser_stats = deserialized.stats.as_ref().unwrap();
        assert_eq!(original_stats.games_played, deser_stats.games_played);
    }

    #[test]
    fn test_opening_source_serde() {
        // Test that source serializes to lowercase as expected
        let opening =
            Opening::new("test", "Test", vec![], "fen").with_source(OpeningSource::Lichess);

        let json = serde_json::to_string(&opening).expect("serialization failed");
        assert!(json.contains("\"source\":\"lichess\""));
    }

    #[test]
    fn test_opening_stats_serde_roundtrip() {
        let stats = OpeningStats::new(12345, 0.40, 0.35, 0.25);
        let json = serde_json::to_string(&stats).expect("serialization failed");
        let deserialized: OpeningStats =
            serde_json::from_str(&json).expect("deserialization failed");

        assert_eq!(stats.games_played, deserialized.games_played);
        assert!((stats.white_wins - deserialized.white_wins).abs() < f32::EPSILON);
        assert!((stats.draws - deserialized.draws).abs() < f32::EPSILON);
        assert!((stats.black_wins - deserialized.black_wins).abs() < f32::EPSILON);
    }
}
