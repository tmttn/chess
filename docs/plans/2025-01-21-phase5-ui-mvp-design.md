# Phase 5: UI MVP Design

**Date**: 2025-01-21
**Status**: Approved

## Overview

A web-based UI for the bot arena that provides real-time match viewing, historical game browsing, Elo rankings, and the ability to start new matches. Built with SvelteKit frontend and Rust/Axum backend.

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Real-time data | WebSocket | Live match updates, reuses bot-client patterns |
| Historical data | SQLite + REST API | Persistent storage, queryable, already used by CLI |
| Server | Rust (Axum) | Consistent with bot-arena, type-safe, performant |
| Elo system | Standard K=32 | Simple, well-understood, sufficient for bot comparison |
| Eval display | Bot + Stockfish | Shows both what bot thought and objective analysis |
| Game replay | Full analysis mode | Explore variations, make alternate moves |
| Match control | Full (start from UI) | More useful than view-only |

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Browser (SvelteKit)                   │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐  │
│  │  Dashboard  │  │Game Browser │  │  Match Control  │  │
│  └─────────────┘  └─────────────┘  └─────────────────┘  │
│         │               │                   │           │
│         └───────────────┼───────────────────┘           │
│                         │                               │
│              ┌──────────┴──────────┐                    │
│              │  @tmttn-chess/*     │                    │
│              │  board, game-store  │                    │
│              └──────────┬──────────┘                    │
└─────────────────────────┼───────────────────────────────┘
                          │ HTTP/WebSocket
┌─────────────────────────┼───────────────────────────────┐
│              Bot Arena Server (Axum)                     │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐  │
│  │  REST API   │  │  WebSocket  │  │  Static Files   │  │
│  └──────┬──────┘  └──────┬──────┘  └─────────────────┘  │
│         │                │                              │
│         └────────┬───────┘                              │
│                  │                                      │
│         ┌────────┴────────┐                             │
│         │    SQLite DB    │                             │
│         └────────┬────────┘                             │
│                  │                                      │
│         ┌────────┴────────┐                             │
│         │  Bot Processes  │                             │
│         │  (UCI engines)  │                             │
│         └─────────────────┘                             │
└─────────────────────────────────────────────────────────┘
```

### Components

**Bot Arena Server (`crates/bot-arena-server/`)**
- Axum-based HTTP server
- REST API for historical data
- WebSocket for live match streaming
- Manages bot processes (UCI communication)
- Serves SvelteKit static build

**Bot Arena UI (`apps/web/bot-arena-ui/`)**
- SvelteKit application
- Uses `@tmttn-chess/board` for chess display
- Uses `@tmttn-chess/game-store` for game state
- Builds to static files embedded in server

## UI Pages

### Dashboard (`/`)
- Bot leaderboard: rank, name, Elo, W/L/D, win rate
- Live match panel: real-time board for in-progress matches
- Quick actions: "New Match", "Run Tournament"
- Recent matches: last 10 completed games

### Game Browser (`/games`)
- Filterable match list (by bot, date, opening, result)
- Pagination
- Click to open game detail

### Game Detail (`/games/:id`)
- Interactive board with piece selection and legal moves
- Move list with navigation (click, arrow keys)
- Evaluation graph: dual-line (bot eval + Stockfish)
- Analysis panel:
  - Position evaluation
  - Best move suggestion
  - Variation explorer (alternate lines)
- Opening name display

### New Match (`/match/new`)
- Bot selector dropdowns (white/black)
- Game count input
- Opening selector (optional)
- Movetime setting
- Start button → redirects to live view

### Live Match (`/match/live/:id`)
- Real-time board updates
- Live evaluation graph
- Growing move list
- Match progress (game X of Y, score)

## API Design

### REST Endpoints

```
GET  /api/bots              List all bots with Elo ratings
GET  /api/bots/:name        Single bot stats & match history

GET  /api/matches           Paginated match list (filters: bot, date, result)
GET  /api/matches/:id       Full match detail with moves & evals
POST /api/matches           Start new match

GET  /api/openings          List available openings
GET  /api/openings/:id      Opening details

GET  /api/analysis/:fen     Get Stockfish eval for position
```

### WebSocket Protocol (`/ws`)

Client → Server:
```json
{ "type": "subscribe", "match_id": "uuid" }
{ "type": "unsubscribe", "match_id": "uuid" }
```

Server → Client:
```json
{ "type": "move", "match_id": "...", "uci": "e2e4", "eval": 30 }
{ "type": "game_end", "match_id": "...", "result": "1-0", "game_num": 1 }
{ "type": "match_end", "match_id": "...", "score": "6-4" }
{ "type": "match_started", "match_id": "...", "white": "bot1", "black": "bot2" }
```

## Data Model

### SQLite Schema

```sql
CREATE TABLE bots (
    name TEXT PRIMARY KEY,
    elo_rating INTEGER DEFAULT 1500,
    games_played INTEGER DEFAULT 0,
    wins INTEGER DEFAULT 0,
    losses INTEGER DEFAULT 0,
    draws INTEGER DEFAULT 0
);

CREATE TABLE matches (
    id TEXT PRIMARY KEY,
    white_bot TEXT REFERENCES bots(name),
    black_bot TEXT REFERENCES bots(name),
    games_total INTEGER,
    white_score REAL,
    black_score REAL,
    opening_id TEXT,
    started_at TEXT,
    finished_at TEXT
);

CREATE TABLE games (
    id TEXT PRIMARY KEY,
    match_id TEXT REFERENCES matches(id),
    game_number INTEGER,
    result TEXT,
    opening_name TEXT,
    pgn TEXT
);

CREATE TABLE moves (
    game_id TEXT REFERENCES games(id),
    ply INTEGER,
    uci TEXT,
    san TEXT,
    fen_after TEXT,
    bot_eval INTEGER,
    stockfish_eval INTEGER,
    PRIMARY KEY (game_id, ply)
);
```

### Elo Calculation

Standard Elo with K=32, updated after each game:

```
Expected = 1 / (1 + 10^((opponent_elo - player_elo) / 400))
Actual = 1 (win), 0.5 (draw), 0 (loss)
NewElo = OldElo + K * (Actual - Expected)
```

## Directory Structure

```
crates/
└── bot-arena-server/
    ├── Cargo.toml
    └── src/
        ├── main.rs
        ├── api/           # REST handlers
        ├── ws/            # WebSocket handlers
        ├── db/            # Database operations
        └── match_runner/  # Bot process management

apps/web/
└── bot-arena-ui/
    ├── package.json
    ├── svelte.config.js
    ├── src/
    │   ├── routes/
    │   │   ├── +page.svelte
    │   │   ├── +layout.svelte
    │   │   ├── games/
    │   │   └── match/
    │   └── lib/
    │       ├── api.ts
    │       ├── stores/
    │       └── components/
    └── static/
```

## Dependencies

**Rust (new)**
- `axum` - Web framework
- `tower`, `tower-http` - Middleware
- `tokio-tungstenite` - WebSocket
- `rust-embed` - Static file embedding

**Already available**
- `rusqlite` - Database
- `serde_json` - Serialization
- `tokio` - Async runtime
- `uuid` - Match IDs

**TypeScript**
- `@tmttn-chess/board`
- `@tmttn-chess/game-store`
- `@tmttn-chess/bot-client`
- `chart.js` or `d3` - Evaluation graph

## Build & Deploy

1. Build UI: `cd apps/web/bot-arena-ui && pnpm build`
2. Build server: `cargo build -p bot-arena-server --release`
3. Run: `./bot-arena-server` serves both API and static files
4. Access: `http://localhost:3000`

## Future Enhancements (Phase 6-7)

- Head-to-head comparison matrix
- Opening explorer with statistics
- Bot profile pages with game history
- Live match spectating mode
- Static HTML export for sharing
- Tournament mode with brackets

---

*Last updated: 2025-01-21*
