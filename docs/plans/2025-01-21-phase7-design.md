# Phase 7: Polish & Export

**Date**: 2025-01-21
**Status**: Approved

## Overview

Complete the bot arena with shareable exports, UI polish, documentation, and performance improvements.

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Templating | Askama | Compile-time, type-safe, fast |
| Export format | Self-contained HTML | No dependencies, easy sharing |
| Board rendering | Inline SVG | No JS needed, works offline |
| Optimization approach | Profile first | Measure before optimizing |

## Architecture

Four parallel tracks that can be worked independently:

```
┌─────────────────────────────────────────────────────────────┐
│                     Phase 7: Polish                          │
├──────────────┬──────────────┬──────────────┬────────────────┤
│   Track A    │   Track B    │   Track C    │    Track D     │
│   Exports    │   Presets    │    Docs      │  Performance   │
├──────────────┼──────────────┼──────────────┼────────────────┤
│ HTML reports │ UI dropdown  │ Fix warnings │ Add indexes    │
│ SVG board    │ Auto-fill    │ Clean TODOs  │ Profile APIs   │
│ 3 endpoints  │ API endpoint │ READMEs      │ Engine pool    │
│              │              │ Arch docs    │ Cache headers  │
└──────────────┴──────────────┴──────────────┴────────────────┘
```

## Track A: Shareable HTML Reports

### Purpose

Generate standalone HTML files that display match/game results without needing the API server. Users can share these on forums, archive them, or view offline.

### Export Types

| Export | Contents | Use Case |
|--------|----------|----------|
| Match Report | All games, scores, Elo changes | Share tournament results |
| Game Viewer | Single game with board, moves, eval | Analyze specific game |
| Bot Profile | Stats, Elo history, best openings | Showcase bot performance |

### API Endpoints

```
GET /api/export/match/:id      → HTML file download
GET /api/export/game/:id       → HTML file download
GET /api/export/bot/:name      → HTML file download
```

### Template Structure

```
crates/bot-arena-server/templates/
├── export_match.html
├── export_game.html
├── export_bot.html
└── components/
    ├── board.html      (SVG chess board)
    ├── move_list.html
    └── eval_graph.html (inline SVG chart)
```

## Track B: UI Preset Integration

### Purpose

Expose the existing CLI presets in the match creation form for easier configuration.

### API Addition

```
GET /api/presets → [{ name, games, time_control, description }]
```

### Config Changes

Add description field to presets in arena.toml:

```toml
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

[presets.elo-test]
games = 10000
time_control = "movetime 1000"
description = "Elo calibration (10000 games, 1s/move)"
```

### UI Changes

- Add preset dropdown at top of `/match/new` form
- When preset selected, auto-fill games and movetime fields
- Show "Custom" option to keep manual behavior
- Fields remain editable after preset selection

## Track C: Documentation & Cleanup

### Doc Warnings to Fix

| Crate | Warning | Fix |
|-------|---------|-----|
| chess-analysis | Unclosed HTML tag `value` (x2) | Escape `<value>` in doc comments |
| chess-openings | Unresolved link to `find_by_moves` | Fix link syntax or remove |

### TODOs to Address

| File | TODO | Action |
|------|------|--------|
| `matches.rs` | "Trigger match runner (Phase G)" | Remove (obsolete) |
| `openings.rs` | "lookup ECO code from opening database" | Implement ECO lookup |
| `worker/main.rs` | "Mark match as failed in database" | Add `fail_match()` function |

### Documentation Additions

1. **Crate READMEs**:
   - `bot-arena-server/README.md` - API overview, endpoints
   - `bot-arena-worker/README.md` - Worker architecture, scaling
   - `chess-analysis/README.md` - Analysis API, quality classification

2. **Architecture doc** (`docs/architecture.md`):
   - System diagram
   - Data flow (match creation → execution → results)
   - Component responsibilities

3. **Deployment guide** (`docs/deployment.md`):
   - Running server + worker
   - Configuration options
   - Stockfish setup

## Track D: Performance Optimization

### Database Indexes

```sql
CREATE INDEX IF NOT EXISTS idx_games_match_id ON games(match_id);
CREATE INDEX IF NOT EXISTS idx_moves_game_id ON moves(game_id);
CREATE INDEX IF NOT EXISTS idx_matches_status ON matches(status);
CREATE INDEX IF NOT EXISTS idx_elo_history_bot_name ON elo_history(bot_name);
```

### Engine Pool Improvements

- Make pool size configurable via config
- Add lazy initialization (spawn on first request)
- Add basic metrics logging

### API Optimizations

- Add `Cache-Control` headers for:
  - Completed matches (immutable)
  - Opening list (long cache)
  - Bot stats (short cache)
- Timing logs to identify slow endpoints

## Task Breakdown

### Track A: Shareable HTML Reports (6 tasks)

1. Add Askama templating dependency and setup
2. Create SVG chess board component template
3. Create match export template and endpoint
4. Create game export template and endpoint
5. Create bot profile export template and endpoint
6. Add export buttons to UI pages

### Track B: UI Preset Integration (4 tasks)

1. Add description field to PresetConfig, update arena.toml
2. Create GET /api/presets endpoint
3. Add preset dropdown to match creation form
4. Auto-fill form fields on preset selection

### Track C: Documentation & Cleanup (6 tasks)

1. Fix doc warnings in chess-analysis and chess-openings
2. Remove obsolete TODO, implement ECO lookup
3. Add fail_match() to worker db module
4. Add README.md to bot-arena-server, worker, chess-analysis
5. Create docs/architecture.md
6. Create docs/deployment.md

### Track D: Performance Optimization (4 tasks)

1. Add database indexes for common queries
2. Add timing logs to API endpoints, identify slow queries
3. Make engine pool size configurable, add lazy init
4. Add Cache-Control headers for appropriate endpoints

**Total: 20 tasks**

## Testing Strategy

**Export tests:**
- Unit tests for template rendering
- Verify HTML is valid and self-contained
- Test with various game states (in-progress, completed, drawn)

**Preset tests:**
- API returns all configured presets
- UI correctly auto-fills form
- Custom values override preset

**Performance tests:**
- Before/after timing comparisons
- Verify indexes are used (EXPLAIN QUERY PLAN)

## Dependencies

**New Rust crates:**
- `askama` - HTML templating

**No new frontend dependencies**

---

*Last updated: 2025-01-21*
