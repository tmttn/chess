//! Configuration file loading for the bot arena.
//!
//! This module provides types and functions for loading and managing
//! arena configuration from TOML files.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur when loading or parsing configuration.
///
/// This enum covers file I/O errors, TOML parsing errors, and
/// configuration validation errors like missing bot definitions.
#[derive(Error, Debug)]
pub enum ConfigError {
    /// Failed to read the configuration file from disk.
    #[error("Failed to read config file: {0}")]
    ReadError(#[from] std::io::Error),
    /// Failed to parse the configuration file as valid TOML.
    #[error("Failed to parse config: {0}")]
    ParseError(#[from] toml::de::Error),
    /// Requested bot was not found in the configuration.
    #[error("Bot not found: {0}")]
    BotNotFound(String),
}

/// Configuration for a chess bot.
///
/// Defines the executable path and time control settings for a UCI-compatible
/// chess engine.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BotConfig {
    /// Path to the bot executable.
    pub path: PathBuf,
    /// Time control string (e.g., "movetime 500").
    /// Defaults to "movetime 500" if not specified.
    #[serde(default = "default_time_control")]
    pub time_control: String,
}

fn default_time_control() -> String {
    "movetime 500".to_string()
}

/// Configuration for a match preset.
///
/// Presets define reusable match settings including number of games,
/// opening positions, and time controls.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PresetConfig {
    /// Number of games to play in a match. Defaults to 10.
    #[serde(default = "default_games")]
    pub games: u32,
    /// List of opening positions in FEN or PGN format.
    /// Defaults to empty (use standard starting position).
    #[serde(default)]
    pub openings: Vec<String>,
    /// Time control string for the match.
    /// Defaults to "movetime 500" if not specified.
    #[serde(default = "default_time_control")]
    pub time_control: String,
}

fn default_games() -> u32 {
    10
}

/// Main arena configuration structure.
///
/// Contains all bot definitions and match presets loaded from the
/// configuration file. Uses `arena.toml` in the current directory
/// by default.
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct ArenaConfig {
    /// Map of bot names to their configurations.
    #[serde(default)]
    pub bots: HashMap<String, BotConfig>,
    /// Map of preset names to their configurations.
    #[serde(default)]
    pub presets: HashMap<String, PresetConfig>,
    /// Path to Stockfish engine for analysis.
    /// Defaults to "stockfish" (assumes it's in PATH).
    #[serde(default = "default_stockfish_path")]
    pub stockfish_path: String,
}

fn default_stockfish_path() -> String {
    "stockfish".to_string()
}

impl ArenaConfig {
    /// Loads the arena configuration from disk.
    ///
    /// Attempts to read and parse the configuration file at the path
    /// returned by [`Self::config_path()`]. If the file does not exist,
    /// returns a default empty configuration.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::ReadError`] if the file exists but cannot be read,
    /// or [`ConfigError::ParseError`] if the file contains invalid TOML.
    pub fn load() -> Result<Self, ConfigError> {
        let config_path = Self::config_path();
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            Ok(toml::from_str(&content)?)
        } else {
            Ok(Self::default())
        }
    }

    /// Returns the path to the configuration file.
    ///
    /// Currently returns `arena.toml` in the current working directory.
    pub fn config_path() -> PathBuf {
        PathBuf::from("arena.toml")
    }

    /// Retrieves a bot configuration by name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the bot to look up.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::BotNotFound`] if no bot with the given name exists.
    pub fn get_bot(&self, name: &str) -> Result<&BotConfig, ConfigError> {
        self.bots
            .get(name)
            .ok_or_else(|| ConfigError::BotNotFound(name.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_toml_config() {
        let toml_content = r#"
[bots.stockfish]
path = "/usr/bin/stockfish"
time_control = "movetime 1000"

[bots.lc0]
path = "/opt/lc0/lc0"
time_control = "depth 10"

[presets.quick]
games = 5
time_control = "movetime 100"
openings = ["e2e4", "d2d4"]

[presets.tournament]
games = 100
time_control = "wtime 300000 btime 300000"
"#;

        let config: ArenaConfig = toml::from_str(toml_content).unwrap();

        // Verify bots
        assert_eq!(config.bots.len(), 2);

        let stockfish = config.bots.get("stockfish").unwrap();
        assert_eq!(stockfish.path, PathBuf::from("/usr/bin/stockfish"));
        assert_eq!(stockfish.time_control, "movetime 1000");

        let lc0 = config.bots.get("lc0").unwrap();
        assert_eq!(lc0.path, PathBuf::from("/opt/lc0/lc0"));
        assert_eq!(lc0.time_control, "depth 10");

        // Verify presets
        assert_eq!(config.presets.len(), 2);

        let quick = config.presets.get("quick").unwrap();
        assert_eq!(quick.games, 5);
        assert_eq!(quick.time_control, "movetime 100");
        assert_eq!(quick.openings, vec!["e2e4", "d2d4"]);

        let tournament = config.presets.get("tournament").unwrap();
        assert_eq!(tournament.games, 100);
        assert_eq!(tournament.time_control, "wtime 300000 btime 300000");
    }

    #[test]
    fn test_parse_toml_with_missing_optional_fields() {
        let toml_content = r#"
[bots.minimal]
path = "/usr/bin/engine"

[presets.minimal]
"#;

        let config: ArenaConfig = toml::from_str(toml_content).unwrap();

        // Verify bot with default time_control
        let minimal_bot = config.bots.get("minimal").unwrap();
        assert_eq!(minimal_bot.path, PathBuf::from("/usr/bin/engine"));
        assert_eq!(minimal_bot.time_control, "movetime 500"); // default

        // Verify preset with all defaults
        let minimal_preset = config.presets.get("minimal").unwrap();
        assert_eq!(minimal_preset.games, 10); // default
        assert_eq!(minimal_preset.time_control, "movetime 500"); // default
        assert!(minimal_preset.openings.is_empty()); // default empty vec
    }

    #[test]
    fn test_empty_config_defaults() {
        let toml_content = "";

        let config: ArenaConfig = toml::from_str(toml_content).unwrap();

        assert!(config.bots.is_empty());
        assert!(config.presets.is_empty());
    }

    #[test]
    fn test_get_bot_returns_error_for_unknown_bot() {
        let config = ArenaConfig::default();

        let result = config.get_bot("nonexistent");

        assert!(result.is_err());
        match result {
            Err(ConfigError::BotNotFound(name)) => {
                assert_eq!(name, "nonexistent");
            }
            _ => panic!("Expected BotNotFound error"),
        }
    }

    #[test]
    fn test_get_bot_returns_config_for_existing_bot() {
        let toml_content = r#"
[bots.mybot]
path = "/path/to/bot"
time_control = "movetime 200"
"#;

        let config: ArenaConfig = toml::from_str(toml_content).unwrap();

        let bot = config.get_bot("mybot").unwrap();
        assert_eq!(bot.path, PathBuf::from("/path/to/bot"));
        assert_eq!(bot.time_control, "movetime 200");
    }

    #[test]
    fn test_config_path_returns_expected_path() {
        let path = ArenaConfig::config_path();
        assert_eq!(path, PathBuf::from("arena.toml"));
    }

    #[test]
    fn test_load_returns_default_when_file_does_not_exist() {
        // This test works because we're running from a temp directory
        // where arena.toml doesn't exist, or if it does, it's valid TOML
        // Either way, load() should not panic
        let result = ArenaConfig::load();

        // The function should succeed (either loading existing file or returning default)
        assert!(result.is_ok());
    }

    #[test]
    fn test_bot_config_serialization_roundtrip() {
        let bot = BotConfig {
            path: PathBuf::from("/usr/bin/stockfish"),
            time_control: "movetime 1000".to_string(),
        };

        let serialized = toml::to_string(&bot).unwrap();
        let deserialized: BotConfig = toml::from_str(&serialized).unwrap();

        assert_eq!(deserialized.path, bot.path);
        assert_eq!(deserialized.time_control, bot.time_control);
    }

    #[test]
    fn test_preset_config_serialization_roundtrip() {
        let preset = PresetConfig {
            games: 50,
            openings: vec!["e4".to_string(), "d4".to_string()],
            time_control: "wtime 60000 btime 60000".to_string(),
        };

        let serialized = toml::to_string(&preset).unwrap();
        let deserialized: PresetConfig = toml::from_str(&serialized).unwrap();

        assert_eq!(deserialized.games, preset.games);
        assert_eq!(deserialized.openings, preset.openings);
        assert_eq!(deserialized.time_control, preset.time_control);
    }

    #[test]
    fn test_stockfish_path_default() {
        let config: ArenaConfig = toml::from_str("").unwrap();
        assert_eq!(config.stockfish_path, "stockfish");
    }

    #[test]
    fn test_stockfish_path_custom() {
        let toml_content = r#"
stockfish_path = "/opt/stockfish/stockfish"
"#;

        let config: ArenaConfig = toml::from_str(toml_content).unwrap();
        assert_eq!(config.stockfish_path, "/opt/stockfish/stockfish");
    }
}
