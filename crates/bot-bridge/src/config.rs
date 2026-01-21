//! Configuration loading for bot-bridge.

use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default)]
    pub bots: HashMap<String, BotConfig>,
}

fn default_port() -> u16 {
    9999
}

#[derive(Debug, Deserialize)]
pub struct BotConfig {
    pub command: String,
}

impl Config {
    pub async fn load() -> Result<Self, Box<dyn std::error::Error>> {
        // Look for bots.toml in current directory or parent directories
        let paths = ["bots.toml", "../bots.toml", "../../bots.toml"];

        for path in paths {
            if Path::new(path).exists() {
                let content = tokio::fs::read_to_string(path).await?;
                let config: Config = toml::from_str(&content)?;
                println!("Loaded config from {}", path);
                return Ok(config);
            }
        }

        // Return default config if no file found
        println!("No bots.toml found, using defaults");
        Ok(Config {
            port: default_port(),
            bots: HashMap::new(),
        })
    }
}
