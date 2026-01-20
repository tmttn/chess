# Bot Plugin System Design

## Overview

A language-agnostic bot plugin system using the UCI (Universal Chess Interface) protocol with custom extensions for debug output. Bots run as separate processes and communicate with the devtools UI via a WebSocket bridge server.

## Architecture

```
┌─────────────┐     WebSocket      ┌──────────────┐    stdin/stdout    ┌─────────────┐
│  Browser    │◄──────────────────►│ Bot Bridge   │◄──────────────────►│  Bot (any   │
│  (devtools) │      JSON/UCI      │  (Rust)      │      UCI           │  language)  │
└─────────────┘                    └──────────────┘                    └─────────────┘
```

## New Crates

| Crate | Type | Purpose |
|-------|------|---------|
| `uci` | library | Shared UCI protocol parsing and formatting |
| `bot-bridge` | binary | WebSocket server, spawns bot processes, routes UCI |
| `bot-random` | binary | Example bot - picks random legal moves |
| `bot-minimax` | binary | Example bot - minimax search with alpha-beta |

## Extended UCI Protocol

### Standard UCI Flow

```
GUI → uci
BOT ← id name RandomBot
BOT ← id author YourName
BOT ← uciok

GUI → isready
BOT ← readyok

GUI → position fen rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
GUI → go movetime 1000

BOT ← info depth 5 score cp 30 nodes 12000 pv e2e4 e7e5
BOT ← bestmove e2e4
```

### Extension Discovery

New `extensions` command allows bots to declare custom debug capabilities:

```
GUI → extensions
BOT ← extension eval description "Position evaluation breakdown"
BOT ← extension thinking description "Human-readable thought process"
BOT ← extension heatmap description "Square importance visualization"
BOT ← extensionsok
```

### Custom Info Format

Extensions use a structured format within `info string`:

```
BOT ← info string ext:eval {"material": 1.5, "mobility": 0.3, "king_safety": -0.2}
BOT ← info string ext:thinking "Considering knight sacrifice on f7..."
BOT ← info string ext:heatmap {"e4": 0.9, "d4": 0.8, "f7": 0.7}
```

The `ext:<name>` prefix enables parsing. JSON payloads allow structured data. Bots without extensions simply omit the `extensions` command.

## Bridge Server

### WebSocket Protocol

Client-to-server messages:

```json
{"type": "connect", "bot": "random"}
{"type": "uci", "cmd": "position fen ... moves e2e4"}
{"type": "uci", "cmd": "go movetime 1000"}
```

Server-to-client messages:

```json
{"type": "connected", "bot": "random", "session": "abc123"}
{"type": "uci", "line": "info depth 5 score cp 30 pv e2e4"}
{"type": "uci", "line": "bestmove e2e4"}
{"type": "disconnected", "reason": "process exited"}
```

### Bot Configuration

Bots are configured in `bots.toml` at the workspace root:

```toml
[bots.random]
command = "./target/release/bot-random"

[bots.minimax]
command = "./target/release/bot-minimax"

[bots.stockfish]
command = "/usr/local/bin/stockfish"
```

This allows plugging in any UCI-compatible engine.

## Devtools UI Integration

### New Components

1. **BotPanel.svelte** - Main bot control panel
   - Dropdown to select bot for white/black (or "Human")
   - Connect/disconnect buttons
   - Connection status indicator

2. **BotDebugPanel.svelte** - Live search visualization
   - Current depth, score, nodes, nps
   - Principal variation (PV) displayed as arrows
   - Dynamic extension panels based on bot capabilities

### Auto-play Flow

1. User assigns "minimax" bot to black
2. User plays e4 (white)
3. Store detects: black's turn + bot assigned
4. Sends: `position fen ... moves e2e4`
5. Sends: `go movetime 2000`
6. Bot streams info → UI updates live
7. Bot sends `bestmove e7e5`
8. Store executes move, plays sound
9. Now white's turn (human) - wait for input

## Directory Structure

```
crates/
  chess-core/       # existing - core types
  chess-engine/     # existing - move generation
  chess-wasm/       # existing - WASM bindings
  uci/              # new - UCI protocol library
  bot-bridge/       # new - WebSocket server
  bot-random/       # new - example random bot
  bot-minimax/      # new - example search bot
```

## Implementation Order

1. `uci` crate - protocol types and parsing
2. `bot-random` - simplest bot to test the protocol
3. `bot-bridge` - WebSocket server
4. UI: BotPanel + basic connection
5. UI: auto-play integration
6. `bot-minimax` - bot with search info
7. UI: BotDebugPanel with extension support
