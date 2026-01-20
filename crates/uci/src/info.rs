//! UCI info command types.

use crate::ExtensionValue;
use serde::{Deserialize, Serialize};

/// Score in centipawns or mate distance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Score {
    /// Centipawn score (100 = 1 pawn advantage).
    Cp(i32),
    /// Mate in N moves (positive = engine winning, negative = engine losing).
    Mate(i32),
}

/// Search information from engine.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct EngineInfo {
    /// Search depth in plies.
    pub depth: Option<u32>,
    /// Selective search depth.
    pub seldepth: Option<u32>,
    /// Score evaluation.
    pub score: Option<Score>,
    /// Nodes searched.
    pub nodes: Option<u64>,
    /// Nodes per second.
    pub nps: Option<u64>,
    /// Time spent in milliseconds.
    pub time: Option<u64>,
    /// Principal variation (best line found).
    pub pv: Vec<String>,
    /// Current move being searched.
    pub currmove: Option<String>,
    /// Current move number.
    pub currmovenumber: Option<u32>,
    /// Hash table usage (per mille).
    pub hashfull: Option<u32>,
    /// Arbitrary string info.
    pub string: Option<String>,
    /// Custom extension data.
    pub extension: Option<(String, ExtensionValue)>,
}

impl EngineInfo {
    /// Create a new empty info.
    pub fn new() -> Self {
        Self::default()
    }

    /// Format as UCI info string.
    pub fn to_uci(&self) -> String {
        let mut parts = vec!["info".to_string()];

        if let Some(d) = self.depth {
            parts.push(format!("depth {}", d));
        }
        if let Some(d) = self.seldepth {
            parts.push(format!("seldepth {}", d));
        }
        if let Some(ref s) = self.score {
            match s {
                Score::Cp(cp) => parts.push(format!("score cp {}", cp)),
                Score::Mate(m) => parts.push(format!("score mate {}", m)),
            }
        }
        if let Some(n) = self.nodes {
            parts.push(format!("nodes {}", n));
        }
        if let Some(n) = self.nps {
            parts.push(format!("nps {}", n));
        }
        if let Some(t) = self.time {
            parts.push(format!("time {}", t));
        }
        if !self.pv.is_empty() {
            parts.push(format!("pv {}", self.pv.join(" ")));
        }
        if let Some(ref m) = self.currmove {
            parts.push(format!("currmove {}", m));
        }
        if let Some(n) = self.currmovenumber {
            parts.push(format!("currmovenumber {}", n));
        }
        if let Some(h) = self.hashfull {
            parts.push(format!("hashfull {}", h));
        }
        if let Some(ref s) = self.string {
            parts.push(format!("string {}", s));
        }
        if let Some((ref name, ref value)) = self.extension {
            let json = serde_json::to_string(value).unwrap_or_default();
            parts.push(format!("string ext:{} {}", name, json));
        }

        parts.join(" ")
    }

    /// Parse UCI info line.
    pub fn parse(line: &str) -> Option<Self> {
        let line = line.trim();
        if !line.starts_with("info") {
            return None;
        }

        let mut info = EngineInfo::new();
        let parts: Vec<&str> = line.split_whitespace().collect();
        let mut i = 1; // Skip "info"

        while i < parts.len() {
            match parts[i] {
                "depth" => {
                    i += 1;
                    if i < parts.len() {
                        info.depth = parts[i].parse().ok();
                    }
                }
                "seldepth" => {
                    i += 1;
                    if i < parts.len() {
                        info.seldepth = parts[i].parse().ok();
                    }
                }
                "score" => {
                    i += 1;
                    if i < parts.len() {
                        match parts[i] {
                            "cp" => {
                                i += 1;
                                if i < parts.len() {
                                    if let Ok(cp) = parts[i].parse() {
                                        info.score = Some(Score::Cp(cp));
                                    }
                                }
                            }
                            "mate" => {
                                i += 1;
                                if i < parts.len() {
                                    if let Ok(m) = parts[i].parse() {
                                        info.score = Some(Score::Mate(m));
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                "nodes" => {
                    i += 1;
                    if i < parts.len() {
                        info.nodes = parts[i].parse().ok();
                    }
                }
                "nps" => {
                    i += 1;
                    if i < parts.len() {
                        info.nps = parts[i].parse().ok();
                    }
                }
                "time" => {
                    i += 1;
                    if i < parts.len() {
                        info.time = parts[i].parse().ok();
                    }
                }
                "pv" => {
                    i += 1;
                    // Collect all remaining moves until another keyword or end
                    while i < parts.len() && !is_info_keyword(parts[i]) {
                        info.pv.push(parts[i].to_string());
                        i += 1;
                    }
                    continue; // Don't increment i again
                }
                "currmove" => {
                    i += 1;
                    if i < parts.len() {
                        info.currmove = Some(parts[i].to_string());
                    }
                }
                "currmovenumber" => {
                    i += 1;
                    if i < parts.len() {
                        info.currmovenumber = parts[i].parse().ok();
                    }
                }
                "hashfull" => {
                    i += 1;
                    if i < parts.len() {
                        info.hashfull = parts[i].parse().ok();
                    }
                }
                "string" => {
                    i += 1;
                    // Rest of line is the string
                    let rest: String = parts[i..].join(" ");

                    // Check for extension format: ext:<name> <json>
                    if rest.starts_with("ext:") {
                        if let Some(space_idx) = rest.find(' ') {
                            let name = rest[4..space_idx].to_string();
                            let json = &rest[space_idx + 1..];
                            if let Ok(value) = serde_json::from_str(json) {
                                info.extension = Some((name, value));
                            }
                        }
                    } else {
                        info.string = Some(rest);
                    }
                    break; // String consumes rest of line
                }
                _ => {}
            }
            i += 1;
        }

        Some(info)
    }
}

fn is_info_keyword(s: &str) -> bool {
    matches!(
        s,
        "depth" | "seldepth" | "score" | "nodes" | "nps" | "time"
        | "pv" | "currmove" | "currmovenumber" | "hashfull" | "string"
    )
}

/// Builder for constructing EngineInfo.
#[derive(Default)]
pub struct InfoBuilder {
    info: EngineInfo,
}

impl InfoBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn depth(mut self, d: u32) -> Self {
        self.info.depth = Some(d);
        self
    }

    pub fn seldepth(mut self, d: u32) -> Self {
        self.info.seldepth = Some(d);
        self
    }

    pub fn score_cp(mut self, cp: i32) -> Self {
        self.info.score = Some(Score::Cp(cp));
        self
    }

    pub fn score_mate(mut self, moves: i32) -> Self {
        self.info.score = Some(Score::Mate(moves));
        self
    }

    pub fn nodes(mut self, n: u64) -> Self {
        self.info.nodes = Some(n);
        self
    }

    pub fn nps(mut self, n: u64) -> Self {
        self.info.nps = Some(n);
        self
    }

    pub fn time(mut self, ms: u64) -> Self {
        self.info.time = Some(ms);
        self
    }

    pub fn pv(mut self, moves: Vec<String>) -> Self {
        self.info.pv = moves;
        self
    }

    pub fn currmove(mut self, mv: &str) -> Self {
        self.info.currmove = Some(mv.to_string());
        self
    }

    pub fn string(mut self, s: &str) -> Self {
        self.info.string = Some(s.to_string());
        self
    }

    pub fn extension(mut self, name: &str, value: ExtensionValue) -> Self {
        self.info.extension = Some((name.to_string(), value));
        self
    }

    pub fn build(self) -> EngineInfo {
        self.info
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn info_to_uci() {
        let info = InfoBuilder::new()
            .depth(10)
            .score_cp(35)
            .nodes(50000)
            .pv(vec!["e2e4".to_string(), "e7e5".to_string()])
            .build();

        let uci = info.to_uci();
        assert!(uci.contains("depth 10"));
        assert!(uci.contains("score cp 35"));
        assert!(uci.contains("nodes 50000"));
        assert!(uci.contains("pv e2e4 e7e5"));
    }

    #[test]
    fn parse_info() {
        let line = "info depth 12 score cp 30 nodes 125000 nps 500000 pv e2e4 e7e5 g1f3";
        let info = EngineInfo::parse(line).unwrap();

        assert_eq!(info.depth, Some(12));
        assert_eq!(info.score, Some(Score::Cp(30)));
        assert_eq!(info.nodes, Some(125000));
        assert_eq!(info.nps, Some(500000));
        assert_eq!(info.pv, vec!["e2e4", "e7e5", "g1f3"]);
    }

    #[test]
    fn parse_mate_score() {
        let line = "info depth 20 score mate 3 pv e2e4";
        let info = EngineInfo::parse(line).unwrap();

        assert_eq!(info.score, Some(Score::Mate(3)));
    }
}
