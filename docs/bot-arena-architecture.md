# Bot Arena Architecture

## System Overview

```
+---------------------------------------------------------------------+
|                         Bot Arena System                             |
+---------------------------------------------------------------------+
|                                                                      |
|  +--------------+     +--------------+     +--------------------+   |
|  |  Svelte UI   |---->|   Axum       |---->|   SQLite Database  |   |
|  |  (Browser)   |<----|   Server     |<----|                    |   |
|  +--------------+     +--------------+     +----------+---------+   |
|         |                    |                        |             |
|         | WebSocket          | Stockfish              | Poll/Write  |
|         v                    v                        v             |
|  +--------------+     +--------------+     +--------------------+   |
|  |  Live Game   |     |   Engine     |     |   Worker Process   |   |
|  |  Updates     |     |   Pool       |     |   (UCI Execution)  |   |
|  +--------------+     +--------------+     +--------------------+   |
|                                                                      |
+---------------------------------------------------------------------+
```

## Components

### Frontend (`apps/web/bot-arena-ui`)

- **Framework**: SvelteKit 2 with Svelte 5 runes
- **Styling**: CSS variables for theming
- **State**: Svelte 5 `$state()` and `$derived()`
- **API**: Fetch-based client in `src/lib/api.ts`
- **WebSocket**: Real-time updates via `src/lib/ws.ts`

### Server (`crates/bot-arena-server`)

- **Framework**: Axum with Tower middleware
- **Database**: SQLite via rusqlite with `Mutex<Connection>` pool
- **WebSocket**: Real-time game move broadcasting via `tokio::sync::broadcast`
- **Analysis**: Stockfish process pool for position evaluation
- **Static Files**: Serves the SvelteKit frontend via `tower_http::services::ServeDir`

Key modules:
- `api/` - REST endpoints for bots, matches, games, analysis, exports
- `ws.rs` - WebSocket handler with subscription-based filtering
- `analysis.rs` - Stockfish engine pool with semaphore-based concurrency
- `watcher.rs` - Database poll for live move updates
- `elo.rs` - Elo rating calculations

### Worker (`crates/bot-arena-worker`)

- **Role**: Execute matches by spawning UCI engine processes
- **Scaling**: Multiple workers can run in parallel via atomic database claims
- **Communication**: Shared SQLite database with `worker_id` tracking
- **Graceful Shutdown**: Handles SIGINT to release claimed matches

Key modules:
- `runner.rs` - Match execution with UCI protocol
- `db.rs` - Database operations (claim, release, finish matches)
- `elo.rs` - Post-match Elo updates

### Shared Libraries

| Crate | Purpose |
|-------|---------|
| `chess-core` | Stable, minimal types (Piece, Color, Square, Move) |
| `chess-engine` | Bitboard-based move generation, position representation |
| `chess-openings` | Opening database, ECO codes |
| `chess-analysis` | Move quality analysis with Stockfish |
| `bot-arena` | CLI for running matches, config parsing |
| `uci` | UCI protocol implementation |
| `bot-minimax` | Minimax search bot implementation |
| `bot-random` | Random move bot for testing |

## Data Flow

### Match Creation
1. User submits match form in UI
2. Server creates match record with `status = 'pending'`
3. Returns match ID to UI

### Match Execution
1. Worker polls for `status = 'pending'` matches
2. Claims match with atomic update (`status = 'running'`, `worker_id = <uuid>`)
3. Spawns UCI engine processes for both bots
4. Plays games, writing moves to database
5. Server's watcher detects new moves and broadcasts via WebSocket
6. Worker updates final scores and Elo ratings
7. Sets `status = 'completed'`

### Position Analysis
1. User requests analysis in game viewer
2. Server assigns request to engine pool (semaphore-limited)
3. Stockfish evaluates position with specified depth
4. Returns score (cp or mate), best move, principal variation

## Database Schema

```sql
-- Bot definitions with Elo ratings
CREATE TABLE bots (
    name TEXT PRIMARY KEY,
    elo_rating INTEGER DEFAULT 1500,
    games_played INTEGER DEFAULT 0,
    wins INTEGER DEFAULT 0,
    losses INTEGER DEFAULT 0,
    draws INTEGER DEFAULT 0,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);

-- Match metadata and scores
CREATE TABLE matches (
    id TEXT PRIMARY KEY,
    white_bot TEXT NOT NULL REFERENCES bots(name),
    black_bot TEXT NOT NULL REFERENCES bots(name),
    games_total INTEGER NOT NULL,
    white_score REAL DEFAULT 0,
    black_score REAL DEFAULT 0,
    opening_id TEXT,
    movetime_ms INTEGER DEFAULT 1000,
    started_at TEXT NOT NULL,
    finished_at TEXT,
    status TEXT DEFAULT 'pending',
    worker_id TEXT
);

-- Individual game records
CREATE TABLE games (
    id TEXT PRIMARY KEY,
    match_id TEXT NOT NULL REFERENCES matches(id),
    game_number INTEGER NOT NULL,
    result TEXT,
    opening_name TEXT,
    pgn TEXT,
    started_at TEXT NOT NULL,
    finished_at TEXT
);

-- Move-by-move game data
CREATE TABLE moves (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    game_id TEXT NOT NULL REFERENCES games(id),
    ply INTEGER NOT NULL,
    uci TEXT NOT NULL,
    san TEXT,
    fen_after TEXT NOT NULL,
    bot_eval INTEGER,
    stockfish_eval INTEGER,
    time_ms INTEGER,
    UNIQUE(game_id, ply)
);

-- Elo rating history
CREATE TABLE elo_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    bot_name TEXT NOT NULL REFERENCES bots(name),
    elo_rating INTEGER NOT NULL,
    recorded_at TEXT NOT NULL,
    match_id TEXT REFERENCES matches(id)
);
```

## WebSocket Protocol

Messages use JSON with a `type` field for discrimination:

```typescript
// Client -> Server
{ "type": "subscribe", "match_id": "abc-123" }
{ "type": "unsubscribe", "match_id": "abc-123" }

// Server -> Client
{ "type": "move", "match_id": "abc-123", "uci": "e2e4", "centipawns": 30 }
{ "type": "game_end", "match_id": "abc-123", "result": "1-0", "game_num": 3 }
{ "type": "match_end", "match_id": "abc-123", "score": "5.5-4.5" }
{ "type": "match_started", "match_id": "abc-123", "white": "Bot1", "black": "Bot2" }
```

## Configuration

All configuration in `arena.toml`:

```toml
[bots.minimax]
path = "./target/release/bot-minimax"
time_control = "movetime 500"

[bots.random]
path = "./target/release/bot-random"
time_control = "movetime 100"

[presets.quick]
games = 10
time_control = "movetime 100"
description = "Fast test run (10 games, 100ms/move)"

[presets.standard]
games = 100
time_control = "movetime 500"
description = "Standard comparison (100 games, 500ms/move)"
```

## API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/health` | GET | Health check |
| `/api/bots` | GET | List all bots |
| `/api/bots/:name` | GET | Get bot details |
| `/api/matches` | GET | List matches |
| `/api/matches` | POST | Create new match |
| `/api/matches/:id` | GET | Get match details |
| `/api/games/:id/moves` | GET | Get game moves |
| `/api/analysis` | GET | Analyze position with Stockfish |
| `/api/export/match/:id` | GET | Export match data |
| `/api/export/game/:id` | GET | Export game data |
| `/api/export/bot/:name` | GET | Export bot statistics |
| `/api/openings` | GET | List openings |
| `/api/presets` | GET | List match presets |
| `/api/stats/head-to-head` | GET | Head-to-head statistics |
| `/ws` | WebSocket | Live updates |

## Concurrency Model

- **Server**: Single-threaded Tokio runtime with async handlers
- **Worker**: Tokio runtime with blocking UCI I/O
- **Database**: `Arc<Mutex<Connection>>` for thread-safe access
- **Analysis Pool**: `Semaphore` limits concurrent Stockfish processes

## Security Considerations

- CORS enabled for cross-origin requests
- No authentication (local development focus)
- Foreign keys enforced in SQLite
- Worker IDs prevent double-claiming matches
