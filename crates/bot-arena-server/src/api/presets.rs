//! Preset configuration API.
//!
//! This module provides endpoints for retrieving match presets
//! from the arena configuration.

use axum::{extract::State, Json};
use serde::Serialize;

use crate::AppState;

/// A preset returned by the API.
///
/// Contains the preset name, number of games, time control, and description.
#[derive(Debug, Clone, Serialize)]
pub struct PresetResponse {
    /// Name of the preset (key in config).
    pub name: String,
    /// Number of games in a match using this preset.
    pub games: u32,
    /// Time control string (e.g., "movetime 100").
    pub time_control: String,
    /// Human-readable description of the preset.
    pub description: String,
}

/// List all available presets.
///
/// Returns a JSON array of all presets configured in arena.toml.
///
/// # Response
///
/// Returns a JSON array of [`PresetResponse`] objects.
pub async fn list_presets(State(state): State<AppState>) -> Json<Vec<PresetResponse>> {
    let presets: Vec<PresetResponse> = state
        .config
        .presets
        .iter()
        .map(|(name, preset)| PresetResponse {
            name: name.clone(),
            games: preset.games,
            time_control: preset.time_control.clone(),
            description: preset.description.clone(),
        })
        .collect();

    Json(presets)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preset_response_serializes_correctly() {
        let preset = PresetResponse {
            name: "quick".to_string(),
            games: 10,
            time_control: "movetime 100".to_string(),
            description: "Fast test matches".to_string(),
        };

        let json = serde_json::to_string(&preset).unwrap();

        assert!(json.contains("\"name\":\"quick\""));
        assert!(json.contains("\"games\":10"));
        assert!(json.contains("\"time_control\":\"movetime 100\""));
        assert!(json.contains("\"description\":\"Fast test matches\""));
    }

    #[test]
    fn test_preset_response_fields() {
        let preset = PresetResponse {
            name: "tournament".to_string(),
            games: 100,
            time_control: "wtime 300000 btime 300000".to_string(),
            description: "Standard tournament settings".to_string(),
        };

        assert_eq!(preset.name, "tournament");
        assert_eq!(preset.games, 100);
        assert_eq!(preset.time_control, "wtime 300000 btime 300000");
        assert_eq!(preset.description, "Standard tournament settings");
    }

    #[test]
    fn test_preset_response_clone() {
        let preset = PresetResponse {
            name: "test".to_string(),
            games: 5,
            time_control: "movetime 500".to_string(),
            description: "Test preset".to_string(),
        };

        let cloned = preset.clone();

        assert_eq!(cloned.name, preset.name);
        assert_eq!(cloned.games, preset.games);
        assert_eq!(cloned.time_control, preset.time_control);
        assert_eq!(cloned.description, preset.description);
    }

    #[test]
    fn test_preset_response_debug() {
        let preset = PresetResponse {
            name: "debug-test".to_string(),
            games: 1,
            time_control: "movetime 50".to_string(),
            description: "Debug test".to_string(),
        };

        let debug_str = format!("{:?}", preset);

        assert!(debug_str.contains("PresetResponse"));
        assert!(debug_str.contains("debug-test"));
    }
}
