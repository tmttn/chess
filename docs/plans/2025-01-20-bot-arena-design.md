# Bot Arena: Bot Comparison Tool Design

## Overview

A hybrid system for comparing chess bot performance through batch simulations and interactive analysis.

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                        Bot Comparison System                         │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌──────────────┐     ┌──────────────┐     ┌──────────────────────┐ │
│  │   Rust CLI   │     │   Shared     │     │   Comparison UI      │ │
│  │  (bot-arena) │     │  Libraries   │     │   (Svelte App)       │ │
│  │              │     │              │     │                      │ │
│  │ • Batch sims │◄───►│ • chess-core │◄───►│ • Results dashboard  │ │
│  │ • UCI spawn  │     │ • openings   │     │ • Game browser       │ │
│  │ • PGN export │     │ • analysis   │     │ • Search analytics   │ │
│  │ • Stockfish  │     │ • storage    │     │ • Opening explorer   │ │
│  └──────┬───────┘     └──────────────┘     └──────────┬───────────┘ │
│         │                                             │             │
│         ▼                                             ▼             │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │                     Storage Layer                             │  │
│  │  • SQLite DB (aggregate stats, game index)                    │  │
│  │  • JSON files (per-game rich data)                            │  │
│  │  • PGN files (shareable game records)                         │  │
│  └──────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────┘
```

**New crates:**
- `bot-arena` - CLI tool for running simulations
- `chess-openings` - Opening database (built-in + ECO + Lichess + custom)
- `chess-analysis` - Move quality analysis, Stockfish integration

**Shared libraries extracted from existing code:**
- `@chess/board` - Board rendering components
- `@chess/game-store` - Game state management
- `@chess/bot-client` - Bot-bridge WebSocket client

## Rust CLI (`bot-arena`)

### Commands

```bash
# Run a match (multiple games between two bots)
bot-arena match minimax random --games 100 --opening "Italian Game"

# Run a tournament (round-robin between multiple bots)
bot-arena tournament minimax random stockfish --games-per-pair 50

# Run with multiple openings (random selection per game)
bot-arena match minimax random --games 1000 --openings "Italian Game,Sicilian Defense,Queen's Gambit"

# Use presets
bot-arena match minimax random --preset quick
bot-arena match minimax random --preset standard
bot-arena match minimax random --preset thorough
bot-arena match minimax random --preset elo-test

# Deep analysis with Stockfish
bot-arena analyze --game-id abc123 --engine stockfish --depth 20

# Opening management
bot-arena openings list
bot-arena openings search "indian"
bot-arena openings import-eco ./eco.pgn
bot-arena openings fetch-lichess --min-games 100000
```

### Match Flow

1. Load opening(s) from database
2. For each game:
   - Spawn both bot processes
   - Send UCI init (`uci`, `isready`)
   - Play opening moves (forced, bots still analyze each position)
   - Bots play freely from opening position
   - Capture UCI info for every move (depth, score, nodes, time, pv)
   - Detect game end (checkmate, stalemate, draw rules)
   - Write game to JSON + PGN
   - Update SQLite stats
3. Output summary

### Configuration (`arena.toml`)

```toml
[bots.minimax]
path = "./target/release/bot-minimax"
time_control = "movetime 500"

[bots.random]
path = "./target/release/bot-random"

[bots.stockfish]
path = "/usr/local/bin/stockfish"
time_control = "depth 15"

# Built-in presets
[presets.quick]
games = 10
openings = ["Italian Game", "Sicilian Defense"]
time_control = "movetime 100"
analyze = false

[presets.standard]
games = 100
openings = "popular-20"
time_control = "movetime 500"
analyze = true
stockfish_depth = 10

[presets.thorough]
games = 1000
openings = "eco-all"
time_control = "movetime 500"
analyze = true
stockfish_depth = 15

[presets.elo-test]
games = 10000
openings = "popular-50"
time_control = "movetime 1000"
analyze = false
parallel = 4

# Opening sets
[opening-sets.popular-20]
source = "lichess"
count = 20

[opening-sets.eco-all]
source = "eco"
```

## Opening Database (`chess-openings`)

### Data Model

```rust
pub struct Opening {
    pub id: String,           // "italian-game"
    pub name: String,         // "Italian Game"
    pub eco: Option<String>,  // "C50"
    pub moves: Vec<String>,   // ["e2e4", "e7e5", "g1f3", "b8c6", "f1c4"]
    pub fen: String,          // Position after moves
    pub source: OpeningSource,
    pub tags: Vec<String>,    // ["open", "1.e4", "classical"]
    pub stats: Option<OpeningStats>,
}

pub struct OpeningStats {
    pub games_played: u64,
    pub white_wins: f32,
    pub draws: f32,
    pub black_wins: f32,
}

pub enum OpeningSource {
    BuiltIn,
    Eco,
    Lichess,
    Custom,
}
```

### Built-in Openings (~100)

| Category | Examples |
|----------|----------|
| Open Games (1.e4 e5) | Italian, Spanish, Scotch, King's Gambit, Petrov |
| Semi-Open (1.e4 other) | Sicilian, French, Caro-Kann, Pirc, Alekhine |
| Closed (1.d4 d5) | Queen's Gambit (Accepted/Declined), Slav, London |
| Indian (1.d4 Nf6) | King's Indian, Nimzo-Indian, Queen's Indian, Grunfeld |
| Flank | English, Reti, Bird, Larsen |
| Gambits | Evans, Smith-Morra, Budapest, Benko |

### API

```rust
let db = OpeningDatabase::load()?;

db.search("indian")?;           // All Indian defenses
db.by_eco("C50")?;              // By ECO code
db.by_tag("gambit")?;           // All gambits
db.popular(20)?;                // Top 20 by Lichess games
db.random_subset(10)?;          // 10 random openings
db.weighted_random(10)?;        // Weighted by popularity
```

## Analysis System (`chess-analysis`)

### Move Quality Classification

```rust
pub enum MoveQuality {
    Best,           // Matches top engine choice
    Excellent,      // Within 10cp of best
    Good,           // Within 30cp of best
    Inaccuracy,     // 30-100cp worse than best
    Mistake,        // 100-300cp worse than best
    Blunder,        // >300cp worse than best
    Forced,         // Only legal move, or opening book move
}

pub struct MoveAnalysis {
    pub uci: String,
    pub san: String,
    pub quality: MoveQuality,

    // Bot's own evaluation (from UCI info)
    pub bot_eval: Option<Evaluation>,
    pub bot_depth: Option<u32>,
    pub bot_nodes: Option<u64>,
    pub bot_time_ms: Option<u64>,
    pub bot_pv: Vec<String>,

    // Stockfish evaluation (optional deep analysis)
    pub engine_eval: Option<Evaluation>,
    pub engine_best_move: Option<String>,
    pub engine_pv: Vec<String>,
    pub centipawn_loss: Option<i32>,
}

pub struct GameAnalysis {
    pub game_id: String,
    pub white_bot: String,
    pub black_bot: String,
    pub opening: Opening,
    pub result: GameResult,
    pub moves: Vec<MoveAnalysis>,
    pub white_stats: PlayerStats,
    pub black_stats: PlayerStats,
}

pub struct PlayerStats {
    pub avg_centipawn_loss: f32,
    pub blunders: u32,
    pub mistakes: u32,
    pub inaccuracies: u32,
    pub avg_depth: f32,
    pub avg_nodes: u64,
    pub avg_time_ms: u64,
    pub accuracy_percent: f32,
}
```

## Storage Layer

### SQLite Schema

```sql
CREATE TABLE bots (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    path TEXT,
    config TEXT
);

CREATE TABLE games (
    id TEXT PRIMARY KEY,
    white_bot TEXT REFERENCES bots(id),
    black_bot TEXT REFERENCES bots(id),
    opening_id TEXT,
    result TEXT,
    termination TEXT,
    move_count INTEGER,
    created_at TIMESTAMP,
    json_path TEXT,
    pgn_path TEXT
);

CREATE TABLE bot_stats (
    bot_id TEXT REFERENCES bots(id),
    opponent_id TEXT REFERENCES bots(id),
    opening_id TEXT,
    games INTEGER,
    wins INTEGER,
    draws INTEGER,
    losses INTEGER,
    avg_centipawn_loss REAL,
    avg_depth REAL,
    avg_time_ms REAL,
    PRIMARY KEY (bot_id, opponent_id, opening_id)
);

CREATE TABLE elo_ratings (
    bot_id TEXT PRIMARY KEY REFERENCES bots(id),
    rating INTEGER DEFAULT 1500,
    games_played INTEGER DEFAULT 0,
    last_updated TIMESTAMP
);
```

### File Structure

```
data/
  games/
    2025-01-20/
      abc123.json
      abc123.pgn
  arena.db
```

### PGN Format

```pgn
[Event "Bot Arena Match"]
[Site "local"]
[Date "2025.01.20"]
[White "minimax"]
[Black "random"]
[Result "1-0"]
[Opening "Italian Game"]
[ECO "C50"]

1. e4 {book} e5 {book} 2. Nf3 {book} Nc6 {book} 3. Bc4 {book}
Nf6 {d=12 eval=+0.35 nodes=45000 time=500ms}
4. d3 {d=10 eval=+0.42 nodes=38000 time=500ms} Be7?!
{Inaccuracy. d=1 eval=-0.15} ...
```

## Comparison UI

### App Structure

```
apps/web/bot-arena/
├── src/
│   ├── routes/
│   │   ├── +page.svelte           # Dashboard
│   │   ├── matches/
│   │   │   ├── +page.svelte       # Match list
│   │   │   └── [id]/+page.svelte  # Match results
│   │   ├── games/
│   │   │   ├── +page.svelte       # Game browser
│   │   │   └── [id]/+page.svelte  # Game replay
│   │   ├── bots/
│   │   │   ├── +page.svelte       # Bot rankings
│   │   │   └── [id]/+page.svelte  # Bot profile
│   │   ├── openings/
│   │   │   └── +page.svelte       # Opening explorer
│   │   └── live/
│   │       └── +page.svelte       # Live matches
│   └── lib/
└── package.json
```

### Pages

| Page | Purpose |
|------|---------|
| Dashboard | Elo leaderboard, recent matches, win rate charts |
| Matches | List matches, filter by bots/openings, start new |
| Games | Browse games, filter by result/opening/bot |
| Game Replay | Board, evaluation graph, move quality markers |
| Bot Profile | Stats, opening performance, search metrics |
| Opening Explorer | Tree view, success rates per bot per line |
| Live | Run matches via bot-bridge |

### Visualizations

**Evaluation Graph:**
```
+2.0 ┤                          ╭──╮
+1.0 ┤      ╭─╮    ╭──────╮   ╭╯  │
 0.0 ┼──────╯ ╰────╯      ╰───╯   ╰──
-1.0 ┤
     └─────────────────────────────────
      1  5  10  15  20  25  30  35  40
```

**Head-to-Head Matrix:**
```
           │ minimax │ random │ stockfish │
───────────┼─────────┼────────┼───────────┤
minimax    │    -    │ 85-10-5│  5-90-5   │
random     │ 5-10-85 │   -    │  0-98-2   │
stockfish  │ 5-90-5  │ 2-98-0 │    -      │
```

**Opening Explorer:**
```
1.e4 (52% white, 28% draw, 20% black)
├─ 1...e5 (48% / 32% / 20%)
│  ├─ Italian - minimax: 65% win
│  └─ Spanish - minimax: 58% win
└─ 1...c5 Sicilian (55% / 25% / 20%)
```

## Data Flow

**CLI → Storage:** Direct write to JSON, PGN, SQLite

**UI → Storage:** Direct filesystem read (v1), optional REST API (v2)

**UI Live Mode:** Bot-bridge WebSocket, saves completed games to storage

## Implementation Phases

### Phase 1: Core CLI
- `bot-arena` crate with match command
- UCI process spawning and game management
- Basic PGN + JSON output
- Built-in openings (~50)
- SQLite schema and stats

### Phase 2: Analysis
- Self-evaluation capture from UCI info
- Stockfish integration
- Move quality classification
- `bot-arena analyze` command

### Phase 3: Opening Database
- `chess-openings` crate
- ECO file import
- Lichess API integration
- Custom opening files
- Preset system

### Phase 4: Shared Libraries
- Extract `@chess/board`
- Extract `@chess/game-store`
- Extract `@chess/bot-client`

### Phase 5: UI MVP
- Svelte app scaffold
- Dashboard with Elo rankings
- Game browser with replay
- Evaluation graph

### Phase 6: Full Visualization
- Head-to-head matrix
- Opening explorer
- Bot profiles
- Live match mode

### Phase 7: Polish
- Static HTML export
- Additional presets
- Performance optimization
- Documentation
