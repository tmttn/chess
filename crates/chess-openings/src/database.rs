//! Opening database storage and lookup.

use std::collections::HashMap;

use rand::seq::SliceRandom;
use rand::Rng;
use thiserror::Error;

use crate::opening::{Opening, OpeningMove, OpeningSource};

/// Errors that can occur when working with opening databases.
#[derive(Debug, Error)]
pub enum DatabaseError {
    /// Failed to parse the opening database.
    #[error("failed to parse opening database: {0}")]
    ParseError(String),

    /// Failed to read the opening database file.
    #[error("failed to read opening database: {0}")]
    IoError(#[from] std::io::Error),

    /// JSON deserialization error.
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

/// A move database that maps positions to candidate moves.
///
/// This database is used during gameplay to select opening moves based on position.
/// For browsing and searching named openings, use [`OpeningDatabase`] instead.
#[derive(Debug, Clone, Default)]
pub struct MoveDatabase {
    /// Maps position keys (move history as string) to candidate moves.
    positions: HashMap<String, Vec<OpeningMove>>,
}

impl MoveDatabase {
    /// Creates a new empty move database.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true if the database is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.positions.is_empty()
    }

    /// Returns the number of positions in the database.
    #[must_use]
    pub fn len(&self) -> usize {
        self.positions.len()
    }

    /// Adds a position with its candidate moves to the database.
    pub fn add_position(&mut self, position_key: impl Into<String>, moves: Vec<OpeningMove>) {
        self.positions.insert(position_key.into(), moves);
    }

    /// Looks up candidate moves for a position.
    #[must_use]
    pub fn lookup(&self, position_key: &str) -> Option<&[OpeningMove]> {
        self.positions.get(position_key).map(|v| v.as_slice())
    }

    /// Selects a random move from the candidates, weighted by their weights.
    pub fn select_move<R: Rng>(&self, position_key: &str, rng: &mut R) -> Option<&OpeningMove> {
        let moves = self.lookup(position_key)?;
        if moves.is_empty() {
            return None;
        }

        // Calculate total weight
        let total_weight: u32 = moves.iter().map(|m| m.weight).sum();
        if total_weight == 0 {
            // If all weights are zero, select uniformly
            return moves.choose(rng);
        }

        // Weighted random selection
        let mut choice = rng.gen_range(0..total_weight);
        for mv in moves {
            if choice < mv.weight {
                return Some(mv);
            }
            choice -= mv.weight;
        }

        // Fallback (shouldn't happen)
        moves.last()
    }
}

/// A database for storing and searching named chess openings.
///
/// This database stores [`Opening`] structs and provides various methods for
/// searching and filtering them by name, ECO code, tags, source, and popularity.
///
/// For position-based move lookup during gameplay, use [`MoveDatabase`] instead.
#[derive(Debug, Clone, Default)]
pub struct OpeningDatabase {
    /// All openings stored in the database.
    openings: Vec<Opening>,
}

impl OpeningDatabase {
    /// Creates a new empty opening database.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new opening database with the given openings.
    #[must_use]
    pub fn with_openings(openings: Vec<Opening>) -> Self {
        Self { openings }
    }

    /// Returns the number of openings in the database.
    #[must_use]
    pub fn len(&self) -> usize {
        self.openings.len()
    }

    /// Returns true if the database contains no openings.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.openings.is_empty()
    }

    /// Adds an opening to the database.
    pub fn add(&mut self, opening: Opening) {
        self.openings.push(opening);
    }

    /// Returns all openings in the database.
    #[must_use]
    pub fn all(&self) -> &[Opening] {
        &self.openings
    }

    /// Finds an opening by its unique ID.
    ///
    /// Returns `None` if no opening with the given ID exists.
    #[must_use]
    pub fn by_id(&self, id: &str) -> Option<&Opening> {
        self.openings.iter().find(|o| o.id == id)
    }

    /// Finds all openings matching an ECO code prefix.
    ///
    /// For example, `by_eco("C5")` would match "C50", "C51", etc.
    #[must_use]
    pub fn by_eco(&self, eco_prefix: &str) -> Vec<&Opening> {
        self.openings
            .iter()
            .filter(|o| {
                o.eco
                    .as_ref()
                    .is_some_and(|eco| eco.starts_with(eco_prefix))
            })
            .collect()
    }

    /// Finds all openings with a specific tag.
    #[must_use]
    pub fn by_tag(&self, tag: &str) -> Vec<&Opening> {
        self.openings
            .iter()
            .filter(|o| o.tags.iter().any(|t| t == tag))
            .collect()
    }

    /// Finds all openings from a specific source.
    #[must_use]
    pub fn by_source(&self, source: OpeningSource) -> Vec<&Opening> {
        self.openings
            .iter()
            .filter(|o| o.source == source)
            .collect()
    }

    /// Searches for openings by name (case-insensitive substring match).
    #[must_use]
    pub fn search(&self, query: &str) -> Vec<&Opening> {
        let query_lower = query.to_lowercase();
        self.openings
            .iter()
            .filter(|o| o.name.to_lowercase().contains(&query_lower))
            .collect()
    }

    /// Returns the top N openings by games played.
    ///
    /// Openings without statistics are sorted to the end.
    #[must_use]
    pub fn popular(&self, n: usize) -> Vec<&Opening> {
        let mut sorted: Vec<_> = self.openings.iter().collect();
        sorted.sort_by(|a, b| {
            let a_games = a.stats.as_ref().map_or(0, |s| s.games_played);
            let b_games = b.stats.as_ref().map_or(0, |s| s.games_played);
            b_games.cmp(&a_games)
        });
        sorted.truncate(n);
        sorted
    }

    /// Returns N random openings from the database.
    ///
    /// If `n` is greater than or equal to the database size, returns all openings
    /// in random order.
    pub fn random_subset<R: Rng>(&self, n: usize, rng: &mut R) -> Vec<&Opening> {
        let mut indices: Vec<usize> = (0..self.openings.len()).collect();
        indices.shuffle(rng);
        indices
            .into_iter()
            .take(n)
            .map(|i| &self.openings[i])
            .collect()
    }

    /// Returns N openings selected randomly, weighted by popularity (games played).
    ///
    /// Openings with more games played are more likely to be selected.
    /// Openings without statistics are assigned a weight of 1.
    ///
    /// If `n` is greater than or equal to the database size, returns all openings.
    pub fn weighted_random<R: Rng>(&self, n: usize, rng: &mut R) -> Vec<&Opening> {
        if self.openings.is_empty() || n == 0 {
            return Vec::new();
        }

        let n = n.min(self.openings.len());
        let mut result = Vec::with_capacity(n);
        let mut available: Vec<(usize, u64)> = self
            .openings
            .iter()
            .enumerate()
            .map(|(i, o)| {
                let weight = o.stats.as_ref().map_or(1, |s| s.games_played.max(1));
                (i, weight)
            })
            .collect();

        for _ in 0..n {
            let total_weight: u64 = available.iter().map(|(_, w)| *w).sum();
            if total_weight == 0 {
                break;
            }

            let mut choice = rng.gen_range(0..total_weight);
            let mut selected_idx = 0;

            for (i, (_, weight)) in available.iter().enumerate() {
                if choice < *weight {
                    selected_idx = i;
                    break;
                }
                choice -= *weight;
            }

            let (opening_idx, _) = available.remove(selected_idx);
            result.push(&self.openings[opening_idx]);
        }

        result
    }

    /// Filters openings using a custom predicate.
    #[must_use]
    pub fn filter<F>(&self, predicate: F) -> Vec<&Opening>
    where
        F: Fn(&Opening) -> bool,
    {
        self.openings.iter().filter(|o| predicate(o)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_move_database() {
        let db = MoveDatabase::new();
        assert!(db.is_empty());
        assert_eq!(db.len(), 0);
        assert!(db.lookup("e2e4").is_none());
    }

    #[test]
    fn test_move_database_add_and_lookup() {
        let mut db = MoveDatabase::new();
        db.add_position(
            "",
            vec![OpeningMove::new("e2e4", 100), OpeningMove::new("d2d4", 80)],
        );

        assert!(!db.is_empty());
        assert_eq!(db.len(), 1);

        let moves = db.lookup("").unwrap();
        assert_eq!(moves.len(), 2);
        assert_eq!(moves[0].uci, "e2e4");
    }

    #[test]
    fn test_move_database_select_move() {
        let mut db = MoveDatabase::new();
        db.add_position("", vec![OpeningMove::new("e2e4", 100)]);

        let mut rng = rand::thread_rng();
        let selected = db.select_move("", &mut rng).unwrap();
        assert_eq!(selected.uci, "e2e4");
    }

    // ===== OpeningDatabase Tests =====

    use crate::opening::{OpeningStats, STARTING_FEN};

    /// Helper function to create test openings.
    fn create_test_openings() -> Vec<Opening> {
        vec![
            Opening::new(
                "italian",
                "Italian Game",
                vec![
                    "e2e4".into(),
                    "e7e5".into(),
                    "g1f3".into(),
                    "b8c6".into(),
                    "f1c4".into(),
                ],
                "r1bqkbnr/pppp1ppp/2n5/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R b KQkq - 3 3",
            )
            .with_eco("C50")
            .with_source(OpeningSource::Eco)
            .with_tags(vec!["1.e4".into(), "open".into()])
            .with_stats(OpeningStats::new(100_000, 0.38, 0.30, 0.32)),
            Opening::new(
                "sicilian",
                "Sicilian Defense",
                vec!["e2e4".into(), "c7c5".into()],
                "rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq c6 0 2",
            )
            .with_eco("B20")
            .with_source(OpeningSource::Lichess)
            .with_tags(vec!["1.e4".into(), "asymmetric".into()])
            .with_stats(OpeningStats::new(500_000, 0.35, 0.28, 0.37)),
            Opening::new(
                "french",
                "French Defense",
                vec!["e2e4".into(), "e7e6".into()],
                "rnbqkbnr/pppp1ppp/4p3/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2",
            )
            .with_eco("C00")
            .with_source(OpeningSource::Eco)
            .with_tags(vec!["1.e4".into(), "solid".into()])
            .with_stats(OpeningStats::new(200_000, 0.36, 0.32, 0.32)),
            Opening::new(
                "queens-gambit",
                "Queen's Gambit",
                vec!["d2d4".into(), "d7d5".into(), "c2c4".into()],
                "rnbqkbnr/ppp1pppp/8/3p4/2PP4/8/PP2PPPP/RNBQKBNR b KQkq c3 0 2",
            )
            .with_eco("D06")
            .with_source(OpeningSource::BuiltIn)
            .with_tags(vec!["1.d4".into(), "gambit".into()]),
            Opening::new(
                "london",
                "London System",
                vec!["d2d4".into(), "d7d5".into(), "c1f4".into()],
                "rnbqkbnr/ppp1pppp/8/3p4/3P1B2/8/PPP1PPPP/RN1QKBNR b KQkq - 1 2",
            )
            .with_source(OpeningSource::Custom)
            .with_tags(vec!["1.d4".into(), "system".into()]),
        ]
    }

    #[test]
    fn test_opening_database_new() {
        let db = OpeningDatabase::new();
        assert!(db.is_empty());
        assert_eq!(db.len(), 0);
    }

    #[test]
    fn test_opening_database_with_openings() {
        let openings = create_test_openings();
        let db = OpeningDatabase::with_openings(openings);
        assert!(!db.is_empty());
        assert_eq!(db.len(), 5);
    }

    #[test]
    fn test_opening_database_add() {
        let mut db = OpeningDatabase::new();
        assert!(db.is_empty());

        db.add(Opening::new(
            "test",
            "Test Opening",
            vec!["e2e4".into()],
            STARTING_FEN,
        ));
        assert!(!db.is_empty());
        assert_eq!(db.len(), 1);

        db.add(Opening::new(
            "test2",
            "Test Opening 2",
            vec!["d2d4".into()],
            STARTING_FEN,
        ));
        assert_eq!(db.len(), 2);
    }

    #[test]
    fn test_opening_database_all() {
        let openings = create_test_openings();
        let db = OpeningDatabase::with_openings(openings);

        let all = db.all();
        assert_eq!(all.len(), 5);
        assert_eq!(all[0].id, "italian");
        assert_eq!(all[4].id, "london");
    }

    #[test]
    fn test_opening_database_by_id() {
        let openings = create_test_openings();
        let db = OpeningDatabase::with_openings(openings);

        let italian = db.by_id("italian");
        assert!(italian.is_some());
        assert_eq!(italian.unwrap().name, "Italian Game");

        let nonexistent = db.by_id("nonexistent");
        assert!(nonexistent.is_none());
    }

    #[test]
    fn test_opening_database_by_eco() {
        let openings = create_test_openings();
        let db = OpeningDatabase::with_openings(openings);

        // Exact match
        let c50 = db.by_eco("C50");
        assert_eq!(c50.len(), 1);
        assert_eq!(c50[0].id, "italian");

        // Prefix match
        let c_openings = db.by_eco("C");
        assert_eq!(c_openings.len(), 2); // Italian (C50) and French (C00)

        // No match
        let e_openings = db.by_eco("E");
        assert!(e_openings.is_empty());

        // Opening without ECO (london) should not match
        let all_d = db.by_eco("D");
        assert_eq!(all_d.len(), 1); // Only Queen's Gambit (D06)
    }

    #[test]
    fn test_opening_database_by_tag() {
        let openings = create_test_openings();
        let db = OpeningDatabase::with_openings(openings);

        let e4_openings = db.by_tag("1.e4");
        assert_eq!(e4_openings.len(), 3); // Italian, Sicilian, French

        let d4_openings = db.by_tag("1.d4");
        assert_eq!(d4_openings.len(), 2); // Queen's Gambit, London

        let gambit_openings = db.by_tag("gambit");
        assert_eq!(gambit_openings.len(), 1);
        assert_eq!(gambit_openings[0].id, "queens-gambit");

        let nonexistent = db.by_tag("nonexistent");
        assert!(nonexistent.is_empty());
    }

    #[test]
    fn test_opening_database_by_source() {
        let openings = create_test_openings();
        let db = OpeningDatabase::with_openings(openings);

        let eco_openings = db.by_source(OpeningSource::Eco);
        assert_eq!(eco_openings.len(), 2); // Italian, French

        let lichess_openings = db.by_source(OpeningSource::Lichess);
        assert_eq!(lichess_openings.len(), 1);
        assert_eq!(lichess_openings[0].id, "sicilian");

        let builtin_openings = db.by_source(OpeningSource::BuiltIn);
        assert_eq!(builtin_openings.len(), 1);
        assert_eq!(builtin_openings[0].id, "queens-gambit");

        let custom_openings = db.by_source(OpeningSource::Custom);
        assert_eq!(custom_openings.len(), 1);
        assert_eq!(custom_openings[0].id, "london");
    }

    #[test]
    fn test_opening_database_search() {
        let openings = create_test_openings();
        let db = OpeningDatabase::with_openings(openings);

        // Case-insensitive search
        let italian = db.search("italian");
        assert_eq!(italian.len(), 1);
        assert_eq!(italian[0].id, "italian");

        let italian_upper = db.search("ITALIAN");
        assert_eq!(italian_upper.len(), 1);

        // Partial match
        let defense = db.search("Defense");
        assert_eq!(defense.len(), 2); // Sicilian Defense, French Defense

        // Multiple words
        let queens = db.search("Queen");
        assert_eq!(queens.len(), 1);
        assert_eq!(queens[0].id, "queens-gambit");

        // No match
        let no_match = db.search("xyz");
        assert!(no_match.is_empty());

        // Empty query matches all (contains empty string)
        let all = db.search("");
        assert_eq!(all.len(), 5);
    }

    #[test]
    fn test_opening_database_popular() {
        let openings = create_test_openings();
        let db = OpeningDatabase::with_openings(openings);

        let top2 = db.popular(2);
        assert_eq!(top2.len(), 2);
        assert_eq!(top2[0].id, "sicilian"); // 500,000 games
        assert_eq!(top2[1].id, "french"); // 200,000 games

        let top1 = db.popular(1);
        assert_eq!(top1.len(), 1);
        assert_eq!(top1[0].id, "sicilian");

        // Request more than available
        let top10 = db.popular(10);
        assert_eq!(top10.len(), 5);

        // Openings without stats should be at the end
        assert_eq!(top10[3].id, "queens-gambit"); // No stats
        assert_eq!(top10[4].id, "london"); // No stats
    }

    #[test]
    fn test_opening_database_random_subset() {
        let openings = create_test_openings();
        let db = OpeningDatabase::with_openings(openings);

        let mut rng = rand::thread_rng();

        // Get 3 random openings
        let random3 = db.random_subset(3, &mut rng);
        assert_eq!(random3.len(), 3);

        // All should be unique
        let ids: Vec<_> = random3.iter().map(|o| &o.id).collect();
        let unique: std::collections::HashSet<_> = ids.iter().collect();
        assert_eq!(unique.len(), 3);

        // Request more than available
        let random10 = db.random_subset(10, &mut rng);
        assert_eq!(random10.len(), 5);
    }

    #[test]
    fn test_opening_database_random_subset_empty() {
        let db = OpeningDatabase::new();
        let mut rng = rand::thread_rng();
        let result = db.random_subset(5, &mut rng);
        assert!(result.is_empty());
    }

    #[test]
    fn test_opening_database_weighted_random() {
        let openings = create_test_openings();
        let db = OpeningDatabase::with_openings(openings);

        let mut rng = rand::thread_rng();

        // Get 3 weighted random openings
        let weighted3 = db.weighted_random(3, &mut rng);
        assert_eq!(weighted3.len(), 3);

        // All should be unique
        let ids: Vec<_> = weighted3.iter().map(|o| &o.id).collect();
        let unique: std::collections::HashSet<_> = ids.iter().collect();
        assert_eq!(unique.len(), 3);
    }

    #[test]
    fn test_opening_database_weighted_random_empty() {
        let db = OpeningDatabase::new();
        let mut rng = rand::thread_rng();

        let result = db.weighted_random(5, &mut rng);
        assert!(result.is_empty());
    }

    #[test]
    fn test_opening_database_weighted_random_zero_count() {
        let openings = create_test_openings();
        let db = OpeningDatabase::with_openings(openings);
        let mut rng = rand::thread_rng();

        let result = db.weighted_random(0, &mut rng);
        assert!(result.is_empty());
    }

    #[test]
    fn test_opening_database_filter() {
        let openings = create_test_openings();
        let db = OpeningDatabase::with_openings(openings);

        // Filter by number of moves
        let short_openings = db.filter(|o| o.moves.len() <= 2);
        assert_eq!(short_openings.len(), 2); // Sicilian (2), French (2)

        // Filter by stats
        let popular = db.filter(|o| o.stats.as_ref().is_some_and(|s| s.games_played >= 200_000));
        assert_eq!(popular.len(), 2); // Sicilian (500k), French (200k)

        // Filter by ECO presence
        let with_eco = db.filter(|o| o.eco.is_some());
        assert_eq!(with_eco.len(), 4); // All except London

        // Complex filter
        let e4_with_high_draw = db.filter(|o| {
            o.tags.contains(&"1.e4".to_string())
                && o.stats.as_ref().is_some_and(|s| s.draws >= 0.30)
        });
        assert_eq!(e4_with_high_draw.len(), 2); // Italian (0.30), French (0.32)
    }

    #[test]
    fn test_opening_database_filter_empty() {
        let db = OpeningDatabase::new();
        let result = db.filter(|_| true);
        assert!(result.is_empty());
    }

    #[test]
    fn test_opening_database_clone() {
        let openings = create_test_openings();
        let db = OpeningDatabase::with_openings(openings);
        let cloned = db.clone();

        assert_eq!(db.len(), cloned.len());
        assert_eq!(
            db.by_id("italian").unwrap().name,
            cloned.by_id("italian").unwrap().name
        );
    }
}
