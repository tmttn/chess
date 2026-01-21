# bot-arena-worker

Match execution worker for the Bot Arena system.

## Overview

Polls the database for pending matches, executes games using UCI chess engines,
and writes results back to the database.

## Running

```bash
cargo run -p bot-arena-worker -- --db data/arena.db --bots-dir ./bots
```

## Architecture

```
┌─────────────┐     ┌─────────────┐
│   Worker    │────►│   SQLite    │
│             │◄────│   Database  │
└──────┬──────┘     └─────────────┘
       │
       ▼
┌─────────────┐
│  UCI Bots   │
│ (processes) │
└─────────────┘
```

## Scaling

Multiple workers can run simultaneously - each claims different matches
using atomic database updates.

## Graceful Shutdown

Send SIGINT (Ctrl+C) to finish current game and release uncompleted matches.
