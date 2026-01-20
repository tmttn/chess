//! Integration tests for chess-analysis crate.
//!
//! These tests require Stockfish to be installed and available in PATH.
//! Run with: `cargo test -p chess-analysis --test integration -- --ignored`

use chess_analysis::{AnalysisConfig, AnalysisEngine, GameAnalyzer, MoveInput, MoveQuality};

/// Check if Stockfish is available in PATH.
fn stockfish_available() -> bool {
    std::process::Command::new("stockfish")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok()
}

#[test]
#[ignore = "requires Stockfish"]
fn test_engine_basic_analysis() {
    if !stockfish_available() {
        eprintln!("Skipping test: Stockfish not available");
        return;
    }

    // Create AnalysisEngine with "stockfish"
    let mut engine = AnalysisEngine::new("stockfish").expect("Failed to create AnalysisEngine");

    // Verify engine name contains "Stockfish"
    let name = engine.name();
    assert!(
        name.to_lowercase().contains("stockfish"),
        "Engine name should contain 'Stockfish', got: {}",
        name
    );

    // Analyze starting position at depth 10
    let starting_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let analysis = engine
        .analyze_fen(starting_fen, 10)
        .expect("Failed to analyze starting position");

    // Verify best_move is not empty
    assert!(
        !analysis.best_move.is_empty(),
        "Best move should not be empty"
    );

    // Verify depth >= 10
    assert!(
        analysis.depth >= 10,
        "Search depth should be at least 10, got: {}",
        analysis.depth
    );
}

#[test]
#[ignore = "requires Stockfish"]
fn test_scholars_mate_game_analysis() {
    if !stockfish_available() {
        eprintln!("Skipping test: Stockfish not available");
        return;
    }

    // Scholar's mate: 1.e4 e5 2.Qh5 Nc6 3.Bc4 Nf6?? 4.Qxf7#
    // In UCI notation:
    // 1. e2e4 e7e5
    // 2. d1h5 b8c6
    // 3. f1c4 g8f6?? (Nf6 is a blunder - allows Qxf7#)
    // 4. h5f7# (checkmate)

    let moves = vec![
        MoveInput {
            uci: "e2e4".to_string(),
            bot_eval_cp: Some(30),
            bot_eval_mate: None,
            bot_depth: Some(10),
            bot_nodes: Some(50000),
            bot_time_ms: Some(100),
            bot_pv: vec!["e2e4".to_string()],
        },
        MoveInput {
            uci: "e7e5".to_string(),
            bot_eval_cp: Some(-30),
            bot_eval_mate: None,
            bot_depth: Some(10),
            bot_nodes: Some(50000),
            bot_time_ms: Some(100),
            bot_pv: vec!["e7e5".to_string()],
        },
        MoveInput {
            uci: "d1h5".to_string(),
            bot_eval_cp: Some(50),
            bot_eval_mate: None,
            bot_depth: Some(10),
            bot_nodes: Some(50000),
            bot_time_ms: Some(100),
            bot_pv: vec!["d1h5".to_string()],
        },
        MoveInput {
            uci: "b8c6".to_string(),
            bot_eval_cp: Some(-50),
            bot_eval_mate: None,
            bot_depth: Some(10),
            bot_nodes: Some(50000),
            bot_time_ms: Some(100),
            bot_pv: vec!["b8c6".to_string()],
        },
        MoveInput {
            uci: "f1c4".to_string(),
            bot_eval_cp: Some(100),
            bot_eval_mate: None,
            bot_depth: Some(10),
            bot_nodes: Some(50000),
            bot_time_ms: Some(100),
            bot_pv: vec!["f1c4".to_string()],
        },
        // Nf6?? - This is the blunder that allows Qxf7#
        MoveInput {
            uci: "g8f6".to_string(),
            bot_eval_cp: Some(-100),
            bot_eval_mate: None,
            bot_depth: Some(10),
            bot_nodes: Some(50000),
            bot_time_ms: Some(100),
            bot_pv: vec!["g8f6".to_string()],
        },
        // Qxf7# - Checkmate
        MoveInput {
            uci: "h5f7".to_string(),
            bot_eval_cp: None,
            bot_eval_mate: Some(0),
            bot_depth: Some(10),
            bot_nodes: Some(50000),
            bot_time_ms: Some(100),
            bot_pv: vec!["h5f7".to_string()],
        },
    ];

    // Use AnalysisConfig with depth 12
    let config = AnalysisConfig {
        depth: 12,
        opening_book_moves: 0,
    };

    let mut analyzer =
        GameAnalyzer::new("stockfish", config).expect("Failed to create GameAnalyzer");

    let analysis = analyzer
        .analyze_game("scholars_mate", "white_bot", "black_bot", &moves, "1-0")
        .expect("Failed to analyze game");

    // Find the g8f6 (Nf6) move - it's move index 5 (0-indexed)
    let nf6_move = &analysis.moves[5];
    assert_eq!(nf6_move.uci, "g8f6", "Move 6 should be g8f6 (Nf6)");

    // Verify that g8f6 (Nf6) is classified as a Blunder
    assert_eq!(
        nf6_move.quality,
        MoveQuality::Blunder,
        "Nf6 should be classified as a Blunder, got: {:?}. CP loss: {:?}",
        nf6_move.quality,
        nf6_move.centipawn_loss
    );
}
