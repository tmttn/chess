# Bot Arena Deployment Guide

## Prerequisites

- Rust 1.70+ (`rustup update stable`)
- Node.js 18+ and pnpm
- Stockfish (optional, for analysis)

## Building

### Backend

```bash
cargo build --release -p bot-arena-server -p bot-arena-worker
```

Binaries will be in `target/release/`:
- `bot-arena-server` - HTTP/WebSocket server
- `bot-arena-worker` - Match execution worker

### Bots

```bash
cargo build --release -p bot-minimax -p bot-random
```

### Frontend

```bash
cd apps/web/bot-arena-ui
pnpm install
pnpm build
```

Static files will be in `apps/web/bot-arena-ui/build/`.

## Configuration

Create `arena.toml` in your working directory:

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

[presets.thorough]
games = 1000
time_control = "movetime 500"
description = "Deep analysis (1000 games, 500ms/move)"
```

## Running

### Development

```bash
# Terminal 1: Server
cargo run -p bot-arena-server

# Terminal 2: Worker
cargo run -p bot-arena-worker
```

The server runs on `http://localhost:3000` by default.

To enable Stockfish analysis:

```bash
STOCKFISH_PATH=/usr/local/bin/stockfish cargo run -p bot-arena-server
```

### Production

```bash
# Create data directory
mkdir -p /var/lib/bot-arena

# Server
./target/release/bot-arena-server

# Worker(s)
./target/release/bot-arena-worker --db /var/lib/bot-arena/arena.db --bots-dir /opt/bots
```

### Worker Arguments

```
bot-arena-worker [OPTIONS]

Options:
    --db <PATH>           Path to SQLite database [default: data/arena.db]
    --poll-interval <MS>  Poll interval in milliseconds [default: 1000]
    --bots-dir <PATH>     Directory containing bot executables [default: bots]
```

## Stockfish Setup

For position analysis, install Stockfish:

```bash
# macOS
brew install stockfish

# Ubuntu/Debian
apt install stockfish

# Arch Linux
pacman -S stockfish
```

Set the `STOCKFISH_PATH` environment variable:

```bash
export STOCKFISH_PATH=$(which stockfish)
```

The server will auto-detect Stockfish if `STOCKFISH_PATH` is set.

## Scaling Workers

Multiple workers can run simultaneously to parallelize match execution:

```bash
# Start 4 workers
for i in {1..4}; do
  ./target/release/bot-arena-worker --db data/arena.db --bots-dir ./bots &
done
```

Each worker:
1. Generates a unique UUID
2. Atomically claims pending matches using the UUID
3. Executes matches independently
4. Releases claimed matches on graceful shutdown (SIGINT)

## Static File Serving

The server serves static files from the `static/` directory by default. To use the SvelteKit build:

```bash
# Copy build output to static directory
cp -r apps/web/bot-arena-ui/build/* static/

# Or symlink
ln -s apps/web/bot-arena-ui/build static
```

## Database Management

The server automatically creates the database and schema on first run.

### WAL Mode (Recommended)

For better concurrent read/write performance:

```bash
sqlite3 data/arena.db "PRAGMA journal_mode=WAL;"
```

### Backup

```bash
sqlite3 data/arena.db ".backup backup.db"
```

### Reset

```bash
rm data/arena.db
# Restart server to recreate
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `STOCKFISH_PATH` | Path to Stockfish executable | (disabled) |
| `RUST_LOG` | Log level (e.g., `info`, `debug`) | `info` |

## Troubleshooting

### "Database is locked"

SQLite has one writer at a time. Solutions:
- Enable WAL mode: `PRAGMA journal_mode=WAL;`
- Reduce poll interval for workers
- Use a single worker for low-volume scenarios

### "Engine not found"

- Verify bot path in `arena.toml`
- Check executable permissions: `chmod +x ./bots/minimax`
- Use absolute paths in configuration

### "Analysis timeout"

- Reduce analysis depth in requests
- Increase engine pool size (default: 2)
- Check Stockfish installation: `stockfish` then `uci`

### Worker not picking up matches

- Check database path matches between server and worker
- Verify match status is `pending`: `sqlite3 data/arena.db "SELECT * FROM matches WHERE status='pending'"`
- Check worker logs for errors

### WebSocket connection failing

- Verify server is running on expected port
- Check CORS configuration for cross-origin requests
- Inspect browser console for connection errors

## Monitoring

### Logs

```bash
# Server with debug logging
RUST_LOG=debug ./target/release/bot-arena-server

# Worker with info logging
RUST_LOG=info ./target/release/bot-arena-worker
```

### Health Check

```bash
curl http://localhost:3000/health
# Returns: ok
```

### Database Inspection

```bash
# List pending matches
sqlite3 data/arena.db "SELECT id, white_bot, black_bot, status FROM matches"

# Check bot Elo ratings
sqlite3 data/arena.db "SELECT name, elo_rating, games_played FROM bots"
```
