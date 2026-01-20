//! Bot Arena - A chess engine comparison tool.
//!
//! This crate provides functionality to run matches between UCI-compatible
//! chess engines, track statistics, and export game records in various formats.
//!
//! # Modules
//!
//! - [`uci_client`] - UCI protocol client for communicating with chess engines
//! - [`game_runner`] - Game execution logic for running matches
//! - [`storage`] - SQLite storage for game results and statistics
//! - [`pgn`] - PGN file generation
//! - [`json_output`] - JSON file generation with search information

pub mod config;
pub mod game_runner;
pub mod json_output;
pub mod pgn;
pub mod storage;
pub mod uci_client;
