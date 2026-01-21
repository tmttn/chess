# Phase 6: Match Execution, Analysis & Enhanced Features

**Date**: 2025-01-21
**Status**: Approved

## Overview

Make the bot arena fully functional from the web UI - start matches, watch them live, and analyze completed games with Stockfish. Add enhanced viewing features including opening explorer, head-to-head matrix, and bot profiles.

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Execution model | Worker process | Better isolation, can scale workers independently |
| Communication | Shared SQLite + broadcast | Simple, worker is stateless, can crash without data loss |
| Analysis | On-demand via API | Lower resource usage, only analyze what users request |
| Sequence | Execution → Analysis → Features | Each builds on the previous |

## Architecture

```
┌─────────────────┐     WebSocket      ┌─────────────────┐
│  Browser (UI)   │◄──────────────────►│  bot-arena-     │
│                 │     REST API       │  server         │
└─────────────────┘◄──────────────────►│                 │
                                       └────────┬────────┘
                                                │ SQLite
                                       ┌────────┴────────┐
                                       │    SQLite DB    │
                                       └────────┬────────┘
                                                │ polls/writes
                                       ┌────────┴────────┐
                                       │  bot-arena-     │
                                       │  worker         │
                                       │  (spawns bots)  │
                                       └─────────────────┘
```

## Sub-Phases

### 6A: Match Execution

**Worker Process (`bot-arena-worker`)**

A new crate that:
1. Polls `matches` table for rows with `status = 'pending'`
2. Claims a match by setting `status = 'running'`
3. Spawns bot processes via UCI protocol (reuses existing `uci_client` code)
4. Plays games, writing each move to `moves` table in real-time
5. Updates match scores and bot Elo ratings on completion
6. Sets `status = 'completed'` when done

**Database changes:**

```sql
ALTER TABLE matches ADD COLUMN status TEXT DEFAULT 'pending';
ALTER TABLE matches ADD COLUMN worker_id TEXT;
```

Status values: `pending`, `running`, `completed`, `failed`

**Server-side changes:**

- `POST /api/matches` creates match with `status = 'pending'`
- Background task watches `moves` table for new rows
- Broadcasts move events via existing WebSocket infrastructure

**Worker lifecycle:**
- Run as: `bot-arena-worker --db data/arena.db`
- Can run multiple instances (each claims different matches)
- Graceful shutdown: finish current game, release uncompleted matches

### 6B: Stockfish Analysis

**API Endpoint:**

```
GET /api/analysis?fen=<fen>&depth=20
```

Response:
```json
{
  "fen": "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1",
  "depth": 20,
  "score_cp": 25,
  "score_mate": null,
  "best_move": "e7e5",
  "pv": ["e7e5", "g1f3", "b8c6"]
}
```

**Implementation:**

- Server maintains pool of Stockfish processes (configurable, default 2)
- Request queued if all engines busy
- Timeout after 10 seconds, returns partial result
- Engine reused across requests

**Frontend integration:**

- Game detail page gets "Analyze" button
- Shows eval bar, best move arrow, principal variation
- Analysis updates as user navigates moves

### 6C: Enhanced Features

**Opening Explorer (`/openings`)**

- List all openings from `chess-openings` crate
- Statistics: games played, white/black win rate, draw rate
- Click opening → filtered game list
- Search by ECO code or name

**Head-to-Head Matrix (`/stats`)**

- Grid showing every bot pair's record
- Cell shows: wins-draws-losses, Elo performance
- Click cell → filtered match list
- Sortable by total games or performance

**Bot Profiles (`/bots/:name`)**

- Elo history graph over time
- Win rate by opening (best/worst)
- Recent matches list
- Head-to-head record vs each opponent

**API additions:**

```
GET /api/openings
GET /api/openings/:eco
GET /api/stats/head-to-head
GET /api/bots/:name
GET /api/bots/:name/elo-history
```

## Configuration

```toml
[database]
path = "data/arena.db"

[server]
port = 3000
static_dir = "static"

[worker]
poll_interval_ms = 1000
bots_dir = "bots"

[analysis]
stockfish_path = "stockfish"
pool_size = 2
default_depth = 20
timeout_ms = 10000
```

## Task Breakdown

### 6A: Match Execution (8 tasks)

1. Add status/worker_id columns to matches table
2. Create bot-arena-worker crate scaffold
3. Add match claiming logic with atomic updates
4. Integrate game_runner from bot-arena
5. Write moves to database in real-time
6. Add server-side move watcher + WebSocket broadcast
7. Update Elo after match completion
8. Add worker graceful shutdown

### 6B: Stockfish Analysis (5 tasks)

1. Add Stockfish engine pool to server
2. Create analysis API endpoint
3. Parse UCI output to structured response
4. Add analysis UI to game detail page
5. Add eval bar and best move visualization

### 6C: Enhanced Features (6 tasks)

1. Add opening statistics API
2. Create opening explorer page
3. Add head-to-head matrix API
4. Create stats page with matrix
5. Add bot profile API with Elo history
6. Create bot profile page

**Total: 19 tasks**

## Testing Strategy

**Worker tests:**
- Unit tests for match claiming logic (concurrent claims, failure recovery)
- Integration test with mock bots
- Test graceful shutdown mid-match

**Analysis tests:**
- Unit tests for Stockfish output parsing
- Integration test with actual Stockfish binary
- Pool exhaustion / timeout handling

**Frontend tests:**
- Vitest tests for new API client methods
- Component tests for new pages

## Deployment

```bash
# Terminal 1: Server
./bot-arena-server --db data/arena.db

# Terminal 2: Worker
./bot-arena-worker --db data/arena.db

# Terminal 3: (Optional) Second worker
./bot-arena-worker --db data/arena.db
```

## Dependencies

**New Rust crates:**
- None required (reuses existing UCI, database code)

**External:**
- Stockfish binary for analysis (optional, graceful degradation if missing)

---

*Last updated: 2025-01-21*
