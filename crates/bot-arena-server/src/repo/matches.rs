//! Match repository for database operations.

use crate::db::DbPool;
use crate::models::{Game, Match, Move};
use rusqlite::OptionalExtension;
use rusqlite::Result as SqliteResult;
use uuid::Uuid;

/// Repository for match database operations.
pub struct MatchRepo {
    db: DbPool,
}

/// Filter options for listing matches.
#[derive(Debug)]
pub struct MatchFilter {
    /// Filter by bot name (matches where bot is white or black).
    pub bot: Option<String>,
    /// Maximum number of results to return.
    pub limit: i32,
    /// Number of results to skip.
    pub offset: i32,
}

impl Default for MatchFilter {
    fn default() -> Self {
        Self {
            bot: None,
            limit: 20,
            offset: 0,
        }
    }
}

impl MatchRepo {
    /// Create a new match repository with the given database pool.
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    /// Create a new match in the database.
    ///
    /// Returns the ID of the newly created match.
    pub fn create(
        &self,
        white_bot: &str,
        black_bot: &str,
        games_total: i32,
        movetime_ms: i32,
        opening_id: Option<&str>,
    ) -> SqliteResult<String> {
        let conn = self.db.lock().unwrap();
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO matches (id, white_bot, black_bot, games_total, movetime_ms, opening_id, started_at, status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'pending')",
            (&id, white_bot, black_bot, games_total, movetime_ms, opening_id, &now),
        )?;

        Ok(id)
    }

    /// List matches with optional filtering.
    ///
    /// Results are ordered by start time (most recent first).
    pub fn list(&self, filter: MatchFilter) -> SqliteResult<Vec<Match>> {
        let conn = self.db.lock().unwrap();

        let sql = if filter.bot.is_some() {
            "SELECT id, white_bot, black_bot, games_total, white_score, black_score,
                    opening_id, movetime_ms, started_at, finished_at, status, worker_id
             FROM matches
             WHERE white_bot = ?1 OR black_bot = ?1
             ORDER BY started_at DESC LIMIT ?2 OFFSET ?3"
        } else {
            "SELECT id, white_bot, black_bot, games_total, white_score, black_score,
                    opening_id, movetime_ms, started_at, finished_at, status, worker_id
             FROM matches
             ORDER BY started_at DESC LIMIT ?1 OFFSET ?2"
        };

        let mut stmt = conn.prepare(sql)?;

        let matches = if let Some(ref bot) = filter.bot {
            stmt.query_map(
                [bot, &filter.limit.to_string(), &filter.offset.to_string()],
                Self::map_row,
            )?
        } else {
            stmt.query_map(
                [&filter.limit.to_string(), &filter.offset.to_string()],
                Self::map_row,
            )?
        };

        Ok(matches.filter_map(|r| r.ok()).collect())
    }

    /// Get a match by ID.
    ///
    /// Returns `None` if the match doesn't exist.
    pub fn get(&self, id: &str) -> SqliteResult<Option<Match>> {
        let conn = self.db.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, white_bot, black_bot, games_total, white_score, black_score,
                    opening_id, movetime_ms, started_at, finished_at, status, worker_id
             FROM matches WHERE id = ?1",
        )?;
        stmt.query_row([id], Self::map_row).optional()
    }

    /// Get all games for a match.
    ///
    /// Games are ordered by game number.
    pub fn get_games(&self, match_id: &str) -> SqliteResult<Vec<Game>> {
        let conn = self.db.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, match_id, game_number, result, opening_name, pgn
             FROM games WHERE match_id = ?1 ORDER BY game_number",
        )?;

        let games = stmt
            .query_map([match_id], |row| {
                Ok(Game {
                    id: row.get(0)?,
                    match_id: row.get(1)?,
                    game_number: row.get(2)?,
                    result: row.get(3)?,
                    opening_name: row.get(4)?,
                    pgn: row.get(5)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(games)
    }

    /// Get all moves for a game.
    ///
    /// Moves are ordered by ply number.
    pub fn get_moves(&self, game_id: &str) -> SqliteResult<Vec<Move>> {
        let conn = self.db.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT ply, uci, san, fen_after, bot_eval, stockfish_eval
             FROM moves WHERE game_id = ?1 ORDER BY ply",
        )?;

        let moves = stmt
            .query_map([game_id], |row| {
                Ok(Move {
                    ply: row.get(0)?,
                    uci: row.get(1)?,
                    san: row.get(2)?,
                    fen_after: row.get(3)?,
                    bot_eval: row.get(4)?,
                    stockfish_eval: row.get(5)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(moves)
    }

    fn map_row(row: &rusqlite::Row) -> rusqlite::Result<Match> {
        Ok(Match {
            id: row.get(0)?,
            white_bot: row.get(1)?,
            black_bot: row.get(2)?,
            games_total: row.get(3)?,
            white_score: row.get(4)?,
            black_score: row.get(5)?,
            opening_id: row.get(6)?,
            movetime_ms: row.get(7)?,
            started_at: row.get(8)?,
            finished_at: row.get(9)?,
            status: row.get(10)?,
            worker_id: row.get(11)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::init_db;

    /// Helper to set up test data with bots and matches.
    fn setup_test_data(db: &DbPool) {
        let conn = db.lock().unwrap();

        // Insert test bots
        conn.execute("INSERT INTO bots (name) VALUES ('stockfish')", [])
            .unwrap();
        conn.execute("INSERT INTO bots (name) VALUES ('komodo')", [])
            .unwrap();
        conn.execute("INSERT INTO bots (name) VALUES ('leela')", [])
            .unwrap();
    }

    fn insert_match(db: &DbPool, id: &str, white: &str, black: &str, started_at: &str) {
        let conn = db.lock().unwrap();
        conn.execute(
            "INSERT INTO matches (id, white_bot, black_bot, games_total, started_at, status)
             VALUES (?1, ?2, ?3, 10, ?4, 'pending')",
            [id, white, black, started_at],
        )
        .unwrap();
    }

    fn insert_game(db: &DbPool, id: &str, match_id: &str, game_number: i32, result: Option<&str>) {
        let conn = db.lock().unwrap();
        conn.execute(
            "INSERT INTO games (id, match_id, game_number, result, started_at)
             VALUES (?1, ?2, ?3, ?4, '2025-01-21')",
            rusqlite::params![id, match_id, game_number, result],
        )
        .unwrap();
    }

    fn insert_move(db: &DbPool, game_id: &str, ply: i32, uci: &str, fen: &str) {
        let conn = db.lock().unwrap();
        conn.execute(
            "INSERT INTO moves (game_id, ply, uci, fen_after)
             VALUES (?1, ?2, ?3, ?4)",
            [game_id, &ply.to_string(), uci, fen],
        )
        .unwrap();
    }

    #[test]
    fn test_list_matches_empty() {
        let db = init_db(":memory:").unwrap();
        let repo = MatchRepo::new(db);
        let matches = repo.list(MatchFilter::default()).unwrap();
        assert!(matches.is_empty());
    }

    #[test]
    fn test_list_matches_returns_all() {
        let db = init_db(":memory:").unwrap();
        setup_test_data(&db);

        insert_match(&db, "match1", "stockfish", "komodo", "2025-01-21T10:00:00");
        insert_match(&db, "match2", "stockfish", "leela", "2025-01-21T11:00:00");
        insert_match(&db, "match3", "komodo", "leela", "2025-01-21T12:00:00");

        let repo = MatchRepo::new(db);
        let matches = repo.list(MatchFilter::default()).unwrap();

        assert_eq!(matches.len(), 3);
        // Most recent first
        assert_eq!(matches[0].id, "match3");
        assert_eq!(matches[1].id, "match2");
        assert_eq!(matches[2].id, "match1");
    }

    #[test]
    fn test_list_matches_with_bot_filter() {
        let db = init_db(":memory:").unwrap();
        setup_test_data(&db);

        insert_match(&db, "match1", "stockfish", "komodo", "2025-01-21T10:00:00");
        insert_match(&db, "match2", "stockfish", "leela", "2025-01-21T11:00:00");
        insert_match(&db, "match3", "komodo", "leela", "2025-01-21T12:00:00");

        let repo = MatchRepo::new(db);
        let filter = MatchFilter {
            bot: Some("stockfish".to_string()),
            ..Default::default()
        };
        let matches = repo.list(filter).unwrap();

        assert_eq!(matches.len(), 2);
        // Both matches involve stockfish (as white or black)
        for m in &matches {
            assert!(m.white_bot == "stockfish" || m.black_bot == "stockfish");
        }
    }

    #[test]
    fn test_list_matches_with_limit() {
        let db = init_db(":memory:").unwrap();
        setup_test_data(&db);

        insert_match(&db, "match1", "stockfish", "komodo", "2025-01-21T10:00:00");
        insert_match(&db, "match2", "stockfish", "leela", "2025-01-21T11:00:00");
        insert_match(&db, "match3", "komodo", "leela", "2025-01-21T12:00:00");

        let repo = MatchRepo::new(db);
        let filter = MatchFilter {
            limit: 2,
            ..Default::default()
        };
        let matches = repo.list(filter).unwrap();

        assert_eq!(matches.len(), 2);
        // Most recent first
        assert_eq!(matches[0].id, "match3");
        assert_eq!(matches[1].id, "match2");
    }

    #[test]
    fn test_list_matches_with_offset() {
        let db = init_db(":memory:").unwrap();
        setup_test_data(&db);

        insert_match(&db, "match1", "stockfish", "komodo", "2025-01-21T10:00:00");
        insert_match(&db, "match2", "stockfish", "leela", "2025-01-21T11:00:00");
        insert_match(&db, "match3", "komodo", "leela", "2025-01-21T12:00:00");

        let repo = MatchRepo::new(db);
        let filter = MatchFilter {
            limit: 2,
            offset: 1,
            ..Default::default()
        };
        let matches = repo.list(filter).unwrap();

        assert_eq!(matches.len(), 2);
        // Skipped match3, got match2 and match1
        assert_eq!(matches[0].id, "match2");
        assert_eq!(matches[1].id, "match1");
    }

    #[test]
    fn test_get_match_exists() {
        let db = init_db(":memory:").unwrap();
        setup_test_data(&db);

        insert_match(&db, "match1", "stockfish", "komodo", "2025-01-21T10:00:00");

        let repo = MatchRepo::new(db);
        let result = repo.get("match1").unwrap();

        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.id, "match1");
        assert_eq!(m.white_bot, "stockfish");
        assert_eq!(m.black_bot, "komodo");
        assert_eq!(m.games_total, 10);
        assert_eq!(m.status, "pending");
    }

    #[test]
    fn test_get_match_not_found() {
        let db = init_db(":memory:").unwrap();
        let repo = MatchRepo::new(db);

        let result = repo.get("nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_get_games_for_match() {
        let db = init_db(":memory:").unwrap();
        setup_test_data(&db);

        insert_match(&db, "match1", "stockfish", "komodo", "2025-01-21T10:00:00");
        insert_game(&db, "game1", "match1", 1, Some("1-0"));
        insert_game(&db, "game2", "match1", 2, Some("0-1"));
        insert_game(&db, "game3", "match1", 3, None);

        let repo = MatchRepo::new(db);
        let games = repo.get_games("match1").unwrap();

        assert_eq!(games.len(), 3);
        assert_eq!(games[0].game_number, 1);
        assert_eq!(games[0].result, Some("1-0".to_string()));
        assert_eq!(games[1].game_number, 2);
        assert_eq!(games[1].result, Some("0-1".to_string()));
        assert_eq!(games[2].game_number, 3);
        assert_eq!(games[2].result, None);
    }

    #[test]
    fn test_get_games_empty() {
        let db = init_db(":memory:").unwrap();
        setup_test_data(&db);

        insert_match(&db, "match1", "stockfish", "komodo", "2025-01-21T10:00:00");

        let repo = MatchRepo::new(db);
        let games = repo.get_games("match1").unwrap();

        assert!(games.is_empty());
    }

    #[test]
    fn test_get_moves_for_game() {
        let db = init_db(":memory:").unwrap();
        setup_test_data(&db);

        insert_match(&db, "match1", "stockfish", "komodo", "2025-01-21T10:00:00");
        insert_game(&db, "game1", "match1", 1, None);

        let fen1 = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
        let fen2 = "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq e6 0 2";

        insert_move(&db, "game1", 1, "e2e4", fen1);
        insert_move(&db, "game1", 2, "e7e5", fen2);

        let repo = MatchRepo::new(db);
        let moves = repo.get_moves("game1").unwrap();

        assert_eq!(moves.len(), 2);
        assert_eq!(moves[0].ply, 1);
        assert_eq!(moves[0].uci, "e2e4");
        assert_eq!(moves[0].fen_after, fen1);
        assert_eq!(moves[1].ply, 2);
        assert_eq!(moves[1].uci, "e7e5");
        assert_eq!(moves[1].fen_after, fen2);
    }

    #[test]
    fn test_get_moves_empty() {
        let db = init_db(":memory:").unwrap();
        setup_test_data(&db);

        insert_match(&db, "match1", "stockfish", "komodo", "2025-01-21T10:00:00");
        insert_game(&db, "game1", "match1", 1, None);

        let repo = MatchRepo::new(db);
        let moves = repo.get_moves("game1").unwrap();

        assert!(moves.is_empty());
    }

    #[test]
    fn test_filter_default_values() {
        let filter = MatchFilter::default();
        assert!(filter.bot.is_none());
        assert_eq!(filter.limit, 20);
        assert_eq!(filter.offset, 0);
    }

    #[test]
    fn test_match_fields_mapped_correctly() {
        let db = init_db(":memory:").unwrap();
        setup_test_data(&db);

        // Insert a match with all fields
        {
            let conn = db.lock().unwrap();
            conn.execute(
                "INSERT INTO matches (id, white_bot, black_bot, games_total, white_score, black_score,
                                      opening_id, movetime_ms, started_at, finished_at, status, worker_id)
                 VALUES ('match1', 'stockfish', 'komodo', 10, 5.5, 4.5, 'e4-openings', 2000,
                         '2025-01-21T10:00:00', '2025-01-21T12:00:00', 'completed', 'worker-1')",
                [],
            )
            .unwrap();
        }

        let repo = MatchRepo::new(db);
        let m = repo.get("match1").unwrap().unwrap();

        assert_eq!(m.id, "match1");
        assert_eq!(m.white_bot, "stockfish");
        assert_eq!(m.black_bot, "komodo");
        assert_eq!(m.games_total, 10);
        assert_eq!(m.white_score, 5.5);
        assert_eq!(m.black_score, 4.5);
        assert_eq!(m.opening_id, Some("e4-openings".to_string()));
        assert_eq!(m.movetime_ms, 2000);
        assert_eq!(m.started_at, "2025-01-21T10:00:00");
        assert_eq!(m.finished_at, Some("2025-01-21T12:00:00".to_string()));
        assert_eq!(m.status, "completed");
        assert_eq!(m.worker_id, Some("worker-1".to_string()));
    }

    #[test]
    fn test_game_fields_mapped_correctly() {
        let db = init_db(":memory:").unwrap();
        setup_test_data(&db);

        insert_match(&db, "match1", "stockfish", "komodo", "2025-01-21T10:00:00");

        // Insert a game with all fields
        {
            let conn = db.lock().unwrap();
            conn.execute(
                "INSERT INTO games (id, match_id, game_number, result, opening_name, pgn, started_at)
                 VALUES ('game1', 'match1', 1, '1-0', 'Sicilian Defense', '1. e4 c5', '2025-01-21T10:00:00')",
                [],
            )
            .unwrap();
        }

        let repo = MatchRepo::new(db);
        let games = repo.get_games("match1").unwrap();

        assert_eq!(games.len(), 1);
        let g = &games[0];
        assert_eq!(g.id, "game1");
        assert_eq!(g.match_id, "match1");
        assert_eq!(g.game_number, 1);
        assert_eq!(g.result, Some("1-0".to_string()));
        assert_eq!(g.opening_name, Some("Sicilian Defense".to_string()));
        assert_eq!(g.pgn, Some("1. e4 c5".to_string()));
    }

    #[test]
    fn test_move_fields_mapped_correctly() {
        let db = init_db(":memory:").unwrap();
        setup_test_data(&db);

        insert_match(&db, "match1", "stockfish", "komodo", "2025-01-21T10:00:00");
        insert_game(&db, "game1", "match1", 1, None);

        // Insert a move with all fields
        {
            let conn = db.lock().unwrap();
            conn.execute(
                "INSERT INTO moves (game_id, ply, uci, san, fen_after, bot_eval, stockfish_eval)
                 VALUES ('game1', 1, 'e2e4', 'e4', 'rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1', 25, 30)",
                [],
            )
            .unwrap();
        }

        let repo = MatchRepo::new(db);
        let moves = repo.get_moves("game1").unwrap();

        assert_eq!(moves.len(), 1);
        let m = &moves[0];
        assert_eq!(m.ply, 1);
        assert_eq!(m.uci, "e2e4");
        assert_eq!(m.san, Some("e4".to_string()));
        assert_eq!(
            m.fen_after,
            "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1"
        );
        assert_eq!(m.bot_eval, Some(25));
        assert_eq!(m.stockfish_eval, Some(30));
    }

    #[test]
    fn test_create_match() {
        let db = init_db(":memory:").unwrap();

        // Create bots first (foreign key constraint)
        {
            let conn = db.lock().unwrap();
            conn.execute("INSERT INTO bots (name) VALUES ('bot1')", [])
                .unwrap();
            conn.execute("INSERT INTO bots (name) VALUES ('bot2')", [])
                .unwrap();
        }

        let repo = MatchRepo::new(db);

        let id = repo.create("bot1", "bot2", 10, 1000, None).unwrap();
        assert!(!id.is_empty());

        let match_info = repo.get(&id).unwrap().unwrap();
        assert_eq!(match_info.white_bot, "bot1");
        assert_eq!(match_info.black_bot, "bot2");
        assert_eq!(match_info.games_total, 10);
        assert_eq!(match_info.movetime_ms, 1000);
        assert_eq!(match_info.status, "pending");
        assert!(match_info.opening_id.is_none());
        assert!(match_info.worker_id.is_none());
    }

    #[test]
    fn test_create_match_with_opening() {
        let db = init_db(":memory:").unwrap();

        // Create bots first (foreign key constraint)
        {
            let conn = db.lock().unwrap();
            conn.execute("INSERT INTO bots (name) VALUES ('bot1')", [])
                .unwrap();
            conn.execute("INSERT INTO bots (name) VALUES ('bot2')", [])
                .unwrap();
        }

        let repo = MatchRepo::new(db);

        let id = repo
            .create("bot1", "bot2", 20, 2000, Some("sicilian"))
            .unwrap();
        assert!(!id.is_empty());

        let match_info = repo.get(&id).unwrap().unwrap();
        assert_eq!(match_info.white_bot, "bot1");
        assert_eq!(match_info.black_bot, "bot2");
        assert_eq!(match_info.games_total, 20);
        assert_eq!(match_info.movetime_ms, 2000);
        assert_eq!(match_info.status, "pending");
        assert_eq!(match_info.opening_id, Some("sicilian".to_string()));
    }
}
