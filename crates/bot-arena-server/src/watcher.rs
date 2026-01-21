//! Watches database for new moves and broadcasts via WebSocket.
//!
//! This module provides a background task that polls the database for new moves
//! and broadcasts them to connected WebSocket clients for real-time updates.

use crate::db::DbPool;
use crate::ws::{WsBroadcast, WsMessage};
use std::collections::HashMap;
use tokio::time::{interval, Duration};

/// Watches the database for new moves and broadcasts them via WebSocket.
///
/// This function runs indefinitely, polling the database every 100ms for new moves
/// and sending them to subscribed WebSocket clients.
///
/// # Arguments
///
/// * `db` - Database connection pool for querying moves
/// * `broadcast` - WebSocket broadcast channel for sending updates
///
/// # Behavior
///
/// The watcher tracks the last seen ply for each game and only broadcasts moves
/// that are newer than the previously seen ply. This ensures each move is only
/// broadcast once even if it appears in multiple polling cycles.
pub async fn watch_moves(db: DbPool, broadcast: WsBroadcast) {
    let mut last_move_plies: HashMap<String, i32> = HashMap::new();
    let mut ticker = interval(Duration::from_millis(100));

    loop {
        ticker.tick().await;

        let new_moves = {
            let conn = match db.lock() {
                Ok(c) => c,
                Err(_) => continue,
            };

            let mut stmt = match conn.prepare(
                "SELECT m.game_id, m.ply, m.uci, g.match_id
                 FROM moves m
                 JOIN games g ON m.game_id = g.id
                 ORDER BY m.rowid DESC
                 LIMIT 100",
            ) {
                Ok(s) => s,
                Err(_) => continue,
            };

            let moves: Vec<(String, i32, String, String)> = stmt
                .query_map([], |row| {
                    Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
                })
                .map(|rows| rows.filter_map(|r| r.ok()).collect())
                .unwrap_or_default();

            moves
        };

        for (game_id, ply, uci, match_id) in new_moves {
            let last_ply = last_move_plies.get(&game_id).copied().unwrap_or(-1);
            if ply > last_ply {
                last_move_plies.insert(game_id.clone(), ply);

                // Broadcast to WebSocket clients
                let _ = broadcast.send(WsMessage::Move {
                    match_id,
                    uci,
                    centipawns: None,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::init_db;
    use crate::ws::create_broadcast;

    #[tokio::test]
    async fn test_watch_moves_broadcasts_new_moves() {
        // Create in-memory database with test data
        let db = init_db(":memory:").expect("Failed to init db");
        let broadcast = create_broadcast();
        let mut rx = broadcast.subscribe();

        // Set up test data
        {
            let conn = db.lock().unwrap();
            conn.execute("INSERT INTO bots (name) VALUES (?)", ["white_bot"])
                .unwrap();
            conn.execute("INSERT INTO bots (name) VALUES (?)", ["black_bot"])
                .unwrap();
            conn.execute(
                "INSERT INTO matches (id, white_bot, black_bot, games_total, started_at) VALUES (?, ?, ?, ?, ?)",
                ["match1", "white_bot", "black_bot", "1", "2025-01-21"],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO games (id, match_id, game_number, started_at) VALUES (?, ?, ?, ?)",
                ["game1", "match1", "1", "2025-01-21"],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO moves (game_id, ply, uci, fen_after) VALUES (?, ?, ?, ?)",
                [
                    "game1",
                    "1",
                    "e2e4",
                    "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1",
                ],
            )
            .unwrap();
        }

        // Spawn watcher
        let db_clone = db.clone();
        let broadcast_clone = broadcast.clone();
        let watcher_handle = tokio::spawn(async move {
            watch_moves(db_clone, broadcast_clone).await;
        });

        // Wait for the watcher to pick up the move
        let result = tokio::time::timeout(Duration::from_secs(1), rx.recv()).await;

        // Verify broadcast received
        match result {
            Ok(Ok(WsMessage::Move {
                match_id,
                uci,
                centipawns,
            })) => {
                assert_eq!(match_id, "match1");
                assert_eq!(uci, "e2e4");
                assert!(centipawns.is_none());
            }
            _ => panic!("Expected Move message within timeout"),
        }

        watcher_handle.abort();
    }

    #[tokio::test]
    async fn test_watch_moves_does_not_rebroadcast_same_move() {
        // Create in-memory database with test data
        let db = init_db(":memory:").expect("Failed to init db");
        let broadcast = create_broadcast();
        let mut rx = broadcast.subscribe();

        // Set up test data with a move
        {
            let conn = db.lock().unwrap();
            conn.execute("INSERT INTO bots (name) VALUES (?)", ["white_bot"])
                .unwrap();
            conn.execute("INSERT INTO bots (name) VALUES (?)", ["black_bot"])
                .unwrap();
            conn.execute(
                "INSERT INTO matches (id, white_bot, black_bot, games_total, started_at) VALUES (?, ?, ?, ?, ?)",
                ["match1", "white_bot", "black_bot", "1", "2025-01-21"],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO games (id, match_id, game_number, started_at) VALUES (?, ?, ?, ?)",
                ["game1", "match1", "1", "2025-01-21"],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO moves (game_id, ply, uci, fen_after) VALUES (?, ?, ?, ?)",
                ["game1", "1", "e2e4", "fen1"],
            )
            .unwrap();
        }

        // Spawn watcher
        let db_clone = db.clone();
        let broadcast_clone = broadcast.clone();
        let watcher_handle = tokio::spawn(async move {
            watch_moves(db_clone, broadcast_clone).await;
        });

        // Wait for first broadcast
        let first = tokio::time::timeout(Duration::from_secs(1), rx.recv()).await;
        assert!(first.is_ok());

        // Wait a bit longer - should not receive the same move again
        let second = tokio::time::timeout(Duration::from_millis(300), rx.recv()).await;
        assert!(
            second.is_err(),
            "Should not receive duplicate broadcast for same move"
        );

        watcher_handle.abort();
    }

    #[tokio::test]
    async fn test_watch_moves_broadcasts_newer_ply() {
        // Create in-memory database
        let db = init_db(":memory:").expect("Failed to init db");
        let broadcast = create_broadcast();
        let mut rx = broadcast.subscribe();

        // Set up test data with first move
        {
            let conn = db.lock().unwrap();
            conn.execute("INSERT INTO bots (name) VALUES (?)", ["white_bot"])
                .unwrap();
            conn.execute("INSERT INTO bots (name) VALUES (?)", ["black_bot"])
                .unwrap();
            conn.execute(
                "INSERT INTO matches (id, white_bot, black_bot, games_total, started_at) VALUES (?, ?, ?, ?, ?)",
                ["match1", "white_bot", "black_bot", "1", "2025-01-21"],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO games (id, match_id, game_number, started_at) VALUES (?, ?, ?, ?)",
                ["game1", "match1", "1", "2025-01-21"],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO moves (game_id, ply, uci, fen_after) VALUES (?, ?, ?, ?)",
                ["game1", "1", "e2e4", "fen1"],
            )
            .unwrap();
        }

        // Spawn watcher
        let db_clone = db.clone();
        let broadcast_clone = broadcast.clone();
        let watcher_handle = tokio::spawn(async move {
            watch_moves(db_clone, broadcast_clone).await;
        });

        // Wait for first broadcast
        let first = tokio::time::timeout(Duration::from_secs(1), rx.recv()).await;
        assert!(matches!(first, Ok(Ok(WsMessage::Move { uci, .. })) if uci == "e2e4"));

        // Add a second move
        {
            let conn = db.lock().unwrap();
            conn.execute(
                "INSERT INTO moves (game_id, ply, uci, fen_after) VALUES (?, ?, ?, ?)",
                ["game1", "2", "e7e5", "fen2"],
            )
            .unwrap();
        }

        // Wait for second broadcast
        let second = tokio::time::timeout(Duration::from_secs(1), rx.recv()).await;
        match second {
            Ok(Ok(WsMessage::Move { uci, .. })) => {
                assert_eq!(uci, "e7e5");
            }
            _ => panic!("Expected second Move message"),
        }

        watcher_handle.abort();
    }

    #[test]
    fn test_last_move_plies_tracking() {
        let mut last_move_plies: HashMap<String, i32> = HashMap::new();

        // Initially no entry
        assert_eq!(last_move_plies.get("game1").copied().unwrap_or(-1), -1);

        // Insert ply 1
        last_move_plies.insert("game1".to_string(), 1);
        assert_eq!(last_move_plies.get("game1").copied().unwrap_or(-1), 1);

        // Update to ply 5
        last_move_plies.insert("game1".to_string(), 5);
        assert_eq!(last_move_plies.get("game1").copied().unwrap_or(-1), 5);

        // Different game
        assert_eq!(last_move_plies.get("game2").copied().unwrap_or(-1), -1);
    }
}
