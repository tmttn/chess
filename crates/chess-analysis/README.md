# chess-analysis

Chess game analysis and move quality classification.

## Overview

Provides tools for analyzing chess games:
- Move quality classification (Best, Excellent, Good, Inaccuracy, Mistake, Blunder)
- Stockfish engine integration
- Centipawn loss calculation
- Game statistics

## Usage

```rust
use chess_analysis::{GameAnalyzer, AnalysisConfig};

let analyzer = GameAnalyzer::new(AnalysisConfig::default())?;
let analysis = analyzer.analyze_game(&moves)?;

println!("Accuracy: {:.1}%", analysis.white_stats.accuracy_percent);
```

## Move Quality Thresholds

| Quality | Centipawn Loss |
|---------|----------------|
| Best | 0 (matches engine) |
| Excellent | ≤ 10 |
| Good | ≤ 30 |
| Inaccuracy | 30-100 |
| Mistake | 100-300 |
| Blunder | > 300 |
