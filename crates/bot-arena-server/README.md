# bot-arena-server

REST API server for the Bot Arena chess bot comparison system.

## Overview

Provides HTTP endpoints for:
- Managing bots, matches, and games
- Real-time game updates via WebSocket
- Stockfish position analysis
- HTML export of results

## Running

```bash
cargo run -p bot-arena-server -- --db data/arena.db
```

## API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/bots` | GET | List all bots |
| `/api/bots/:name` | GET | Get bot profile |
| `/api/matches` | GET/POST | List/create matches |
| `/api/games` | GET | List games |
| `/api/games/:id` | GET | Get game details |
| `/api/analysis` | GET | Analyze position |
| `/api/presets` | GET | List match presets |
| `/api/export/*` | GET | Export HTML reports |
| `/ws` | WS | Real-time updates |

## Configuration

See `arena.toml` in project root for configuration options.
