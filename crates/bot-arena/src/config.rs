use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    ReadError(#[from] std::io::Error),
    #[error("Failed to parse config: {0}")]
    ParseError(#[from] toml::de::Error),
    #[error("Bot not found: {0}")]
    BotNotFound(String),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BotConfig {
    pub path: PathBuf,
    #[serde(default = "default_time_control")]
    pub time_control: String,
}

fn default_time_control() -> String {
    "movetime 500".to_string()
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PresetConfig {
    #[serde(default = "default_games")]
    pub games: u32,
    #[serde(default)]
    pub openings: Vec<String>,
    #[serde(default = "default_time_control")]
    pub time_control: String,
}

fn default_games() -> u32 {
    10
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct ArenaConfig {
    #[serde(default)]
    pub bots: HashMap<String, BotConfig>,
    #[serde(default)]
    pub presets: HashMap<String, PresetConfig>,
}

impl ArenaConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let config_path = Self::config_path();
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            Ok(toml::from_str(&content)?)
        } else {
            Ok(Self::default())
        }
    }

    pub fn config_path() -> PathBuf {
        PathBuf::from("arena.toml")
    }

    pub fn get_bot(&self, name: &str) -> Result<&BotConfig, ConfigError> {
        self.bots
            .get(name)
            .ok_or_else(|| ConfigError::BotNotFound(name.to_string()))
    }
}
