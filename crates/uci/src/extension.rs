//! UCI protocol extensions for custom debug info.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Extension declaration from engine.
#[derive(Debug, Clone, PartialEq)]
pub struct Extension {
    /// Extension name (e.g., "eval", "thinking", "heatmap").
    pub name: String,
    /// Human-readable description.
    pub description: String,
}

/// Extension value - flexible JSON-like type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ExtensionValue {
    /// Null value.
    Null,
    /// Boolean value.
    Bool(bool),
    /// Integer value.
    Int(i64),
    /// Float value.
    Float(f64),
    /// String value.
    String(String),
    /// Array of values.
    Array(Vec<ExtensionValue>),
    /// Object/map of values.
    Object(HashMap<String, ExtensionValue>),
}

impl ExtensionValue {
    /// Create an object from key-value pairs.
    pub fn object<I, K, V>(pairs: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<ExtensionValue>,
    {
        ExtensionValue::Object(
            pairs
                .into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
        )
    }

    /// Create an array from values.
    pub fn array<I, V>(values: I) -> Self
    where
        I: IntoIterator<Item = V>,
        V: Into<ExtensionValue>,
    {
        ExtensionValue::Array(values.into_iter().map(Into::into).collect())
    }
}

impl From<bool> for ExtensionValue {
    fn from(v: bool) -> Self {
        ExtensionValue::Bool(v)
    }
}

impl From<i32> for ExtensionValue {
    fn from(v: i32) -> Self {
        ExtensionValue::Int(v as i64)
    }
}

impl From<i64> for ExtensionValue {
    fn from(v: i64) -> Self {
        ExtensionValue::Int(v)
    }
}

impl From<f64> for ExtensionValue {
    fn from(v: f64) -> Self {
        ExtensionValue::Float(v)
    }
}

impl From<&str> for ExtensionValue {
    fn from(v: &str) -> Self {
        ExtensionValue::String(v.to_string())
    }
}

impl From<String> for ExtensionValue {
    fn from(v: String) -> Self {
        ExtensionValue::String(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extension_value_object() {
        let eval = ExtensionValue::object([
            ("material", ExtensionValue::Float(1.5)),
            ("mobility", ExtensionValue::Float(0.3)),
        ]);

        let json = serde_json::to_string(&eval).unwrap();
        assert!(json.contains("material"));
        assert!(json.contains("1.5"));
    }

    #[test]
    fn extension_value_roundtrip() {
        let original = ExtensionValue::object([
            ("score", ExtensionValue::Int(100)),
            ("best_move", ExtensionValue::String("e2e4".to_string())),
        ]);

        let json = serde_json::to_string(&original).unwrap();
        let parsed: ExtensionValue = serde_json::from_str(&json).unwrap();

        assert_eq!(original, parsed);
    }
}
