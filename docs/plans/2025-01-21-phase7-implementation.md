# Phase 7: Polish & Export - Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Complete the bot arena with shareable HTML exports, UI preset integration, documentation, and performance optimizations.

**Architecture:** Four parallel tracks - HTML exports using Askama templates with inline SVG boards, UI presets via new API endpoint, documentation cleanup fixing warnings and adding READMEs, performance via database indexes and caching headers.

**Tech Stack:** Rust/Axum, Askama templates, SQLite, Svelte 5, TypeScript

---

## Track A: Shareable HTML Reports

### Task 1: Add Askama templating dependency

**Files:**
- Modify: `crates/bot-arena-server/Cargo.toml`

**Step 1: Add askama dependency**

```toml
# Add to [dependencies] section
askama = "0.12"
```

**Step 2: Verify it compiles**

Run: `cargo check -p bot-arena-server`
Expected: Compiles without errors

**Step 3: Commit**

```bash
git add crates/bot-arena-server/Cargo.toml
git commit -m "chore(bot-arena-server): add askama templating dependency"
```

---

### Task 2: Create SVG chess board template component

**Files:**
- Create: `crates/bot-arena-server/templates/components/board.html`
- Create: `crates/bot-arena-server/src/templates/mod.rs`
- Create: `crates/bot-arena-server/src/templates/board.rs`
- Modify: `crates/bot-arena-server/src/lib.rs`

**Step 1: Create templates directory structure**

```bash
mkdir -p crates/bot-arena-server/templates/components
mkdir -p crates/bot-arena-server/src/templates
```

**Step 2: Create board.html template**

Create `crates/bot-arena-server/templates/components/board.html`:

```html
<svg viewBox="0 0 400 400" xmlns="http://www.w3.org/2000/svg" style="max-width: 400px;">
  <defs>
    <style>
      .light { fill: #f0d9b5; }
      .dark { fill: #b58863; }
      .piece { font-family: Arial, sans-serif; font-size: 40px; text-anchor: middle; dominant-baseline: middle; }
    </style>
  </defs>

  <!-- Board squares -->
  {% for row in 0..8 %}
  {% for col in 0..8 %}
  <rect x="{{ col * 50 }}" y="{{ row * 50 }}" width="50" height="50"
        class="{% if (row + col) % 2 == 0 %}light{% else %}dark{% endif %}" />
  {% endfor %}
  {% endfor %}

  <!-- Pieces -->
  {% for piece in pieces %}
  <text x="{{ piece.col * 50 + 25 }}" y="{{ piece.row * 50 + 25 }}" class="piece">{{ piece.symbol }}</text>
  {% endfor %}

  <!-- Coordinates -->
  {% for i in 0..8 %}
  <text x="{{ i * 50 + 25 }}" y="395" style="font-size: 10px; text-anchor: middle;">{{ "abcdefgh".chars().nth(i).unwrap() }}</text>
  <text x="5" y="{{ i * 50 + 25 }}" style="font-size: 10px; dominant-baseline: middle;">{{ 8 - i }}</text>
  {% endfor %}
</svg>
```

**Step 3: Create board.rs module**

Create `crates/bot-arena-server/src/templates/board.rs`:

```rust
//! SVG chess board rendering for HTML exports.

use askama::Template;

/// A piece to render on the board.
#[derive(Debug, Clone)]
pub struct PieceView {
    pub row: usize,
    pub col: usize,
    pub symbol: char,
}

/// SVG chess board template.
#[derive(Template)]
#[template(path = "components/board.html")]
pub struct BoardTemplate {
    pub pieces: Vec<PieceView>,
}

impl BoardTemplate {
    /// Create a board from a FEN position string.
    pub fn from_fen(fen: &str) -> Self {
        let mut pieces = Vec::new();
        let board_part = fen.split_whitespace().next().unwrap_or("");

        for (row, rank) in board_part.split('/').enumerate() {
            let mut col = 0;
            for c in rank.chars() {
                if let Some(skip) = c.to_digit(10) {
                    col += skip as usize;
                } else {
                    let symbol = match c {
                        'K' => '♔', 'Q' => '♕', 'R' => '♖', 'B' => '♗', 'N' => '♘', 'P' => '♙',
                        'k' => '♚', 'q' => '♛', 'r' => '♜', 'b' => '♝', 'n' => '♞', 'p' => '♟',
                        _ => continue,
                    };
                    pieces.push(PieceView { row, col, symbol });
                    col += 1;
                }
            }
        }

        Self { pieces }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_board_from_starting_fen() {
        let board = BoardTemplate::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        // 32 pieces in starting position
        assert_eq!(board.pieces.len(), 32);
    }

    #[test]
    fn test_board_from_empty_fen() {
        let board = BoardTemplate::from_fen("8/8/8/8/8/8/8/8 w - - 0 1");
        assert_eq!(board.pieces.len(), 0);
    }

    #[test]
    fn test_board_renders() {
        let board = BoardTemplate::from_fen("8/8/8/8/8/8/8/8 w - - 0 1");
        let html = board.render().unwrap();
        assert!(html.contains("<svg"));
        assert!(html.contains("viewBox"));
    }
}
```

**Step 4: Create templates module**

Create `crates/bot-arena-server/src/templates/mod.rs`:

```rust
//! HTML templates for export functionality.

pub mod board;

pub use board::{BoardTemplate, PieceView};
```

**Step 5: Add templates module to lib.rs**

Add to `crates/bot-arena-server/src/lib.rs`:

```rust
pub mod templates;
```

**Step 6: Run tests**

Run: `cargo test -p bot-arena-server templates`
Expected: All tests pass

**Step 7: Commit**

```bash
git add crates/bot-arena-server/templates/ crates/bot-arena-server/src/templates/
git add crates/bot-arena-server/src/lib.rs
git commit -m "feat(bot-arena-server): add SVG chess board template"
```

---

### Task 3: Create match export template and endpoint

**Files:**
- Create: `crates/bot-arena-server/templates/export_match.html`
- Create: `crates/bot-arena-server/src/templates/match_export.rs`
- Create: `crates/bot-arena-server/src/api/export.rs`
- Modify: `crates/bot-arena-server/src/api/mod.rs`
- Modify: `crates/bot-arena-server/src/templates/mod.rs`

**Step 1: Create match export template**

Create `crates/bot-arena-server/templates/export_match.html`:

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Match: {{ white_bot }} vs {{ black_bot }}</title>
    <style>
        :root { --bg: #1a1a2e; --fg: #eee; --accent: #e94560; --border: #0f3460; }
        * { box-sizing: border-box; margin: 0; padding: 0; }
        body { font-family: system-ui, sans-serif; background: var(--bg); color: var(--fg); padding: 2rem; }
        h1 { margin-bottom: 1rem; }
        .match-info { background: var(--border); padding: 1rem; border-radius: 8px; margin-bottom: 2rem; }
        .score { font-size: 2rem; font-weight: bold; color: var(--accent); }
        .games-table { width: 100%; border-collapse: collapse; margin-top: 1rem; }
        .games-table th, .games-table td { padding: 0.5rem; text-align: left; border-bottom: 1px solid var(--border); }
        .games-table th { color: var(--accent); }
        .result-1-0 { color: #4ade80; }
        .result-0-1 { color: #f87171; }
        .result-draw { color: #fbbf24; }
        footer { margin-top: 2rem; font-size: 0.8rem; color: #888; }
    </style>
</head>
<body>
    <h1>Match Report</h1>

    <div class="match-info">
        <p><strong>{{ white_bot }}</strong> vs <strong>{{ black_bot }}</strong></p>
        <p class="score">{{ white_score }} - {{ black_score }}</p>
        <p>{{ games|length }} games played</p>
        {% if let Some(date) = created_at %}
        <p>Date: {{ date }}</p>
        {% endif %}
    </div>

    <h2>Games</h2>
    <table class="games-table">
        <thead>
            <tr>
                <th>#</th>
                <th>White</th>
                <th>Black</th>
                <th>Result</th>
                <th>Moves</th>
            </tr>
        </thead>
        <tbody>
            {% for game in games %}
            <tr>
                <td>{{ loop.index }}</td>
                <td>{{ game.white }}</td>
                <td>{{ game.black }}</td>
                <td class="{% if game.result == "1-0" %}result-1-0{% elif game.result == "0-1" %}result-0-1{% else %}result-draw{% endif %}">
                    {{ game.result }}
                </td>
                <td>{{ game.move_count }}</td>
            </tr>
            {% endfor %}
        </tbody>
    </table>

    <footer>
        <p>Generated by Bot Arena</p>
    </footer>
</body>
</html>
```

**Step 2: Create match_export.rs**

Create `crates/bot-arena-server/src/templates/match_export.rs`:

```rust
//! Match export template.

use askama::Template;

/// A game summary for the match export.
#[derive(Debug, Clone)]
pub struct GameSummary {
    pub white: String,
    pub black: String,
    pub result: String,
    pub move_count: i32,
}

/// Match export HTML template.
#[derive(Template)]
#[template(path = "export_match.html")]
pub struct MatchExportTemplate {
    pub white_bot: String,
    pub black_bot: String,
    pub white_score: f64,
    pub black_score: f64,
    pub games: Vec<GameSummary>,
    pub created_at: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_export_renders() {
        let template = MatchExportTemplate {
            white_bot: "minimax".to_string(),
            black_bot: "random".to_string(),
            white_score: 2.5,
            black_score: 0.5,
            games: vec![
                GameSummary {
                    white: "minimax".to_string(),
                    black: "random".to_string(),
                    result: "1-0".to_string(),
                    move_count: 40,
                },
            ],
            created_at: Some("2025-01-21".to_string()),
        };

        let html = template.render().unwrap();
        assert!(html.contains("minimax"));
        assert!(html.contains("2.5 - 0.5"));
        assert!(html.contains("1-0"));
    }
}
```

**Step 3: Update templates/mod.rs**

```rust
//! HTML templates for export functionality.

pub mod board;
pub mod match_export;

pub use board::{BoardTemplate, PieceView};
pub use match_export::{MatchExportTemplate, GameSummary};
```

**Step 4: Create export.rs API module**

Create `crates/bot-arena-server/src/api/export.rs`:

```rust
//! Export API endpoints for generating standalone HTML reports.

use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use askama::Template;

use crate::templates::{GameSummary, MatchExportTemplate};
use crate::AppState;

/// Export a match as a standalone HTML file.
pub async fn export_match(
    State(state): State<AppState>,
    Path(match_id): Path<String>,
) -> Result<Response, StatusCode> {
    let conn = state.db.lock().unwrap();

    // Get match info
    let match_row: Result<(String, String, f64, f64, Option<String>), _> = conn.query_row(
        "SELECT white_bot, black_bot, white_score, black_score, created_at FROM matches WHERE id = ?",
        [&match_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
    );

    let (white_bot, black_bot, white_score, black_score, created_at) = match_row
        .map_err(|_| StatusCode::NOT_FOUND)?;

    // Get games
    let mut stmt = conn
        .prepare(
            "SELECT g.id, g.game_number, g.result,
                    (SELECT COUNT(*) FROM moves WHERE game_id = g.id) as move_count
             FROM games g
             WHERE g.match_id = ?
             ORDER BY g.game_number"
        )
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let games: Vec<GameSummary> = stmt
        .query_map([&match_id], |row| {
            let game_number: i32 = row.get(1)?;
            let result: String = row.get(2)?;
            let move_count: i32 = row.get(3)?;

            // Determine who played white/black based on game number
            let (white, black) = if game_number % 2 == 0 {
                (white_bot.clone(), black_bot.clone())
            } else {
                (black_bot.clone(), white_bot.clone())
            };

            Ok(GameSummary { white, black, result, move_count })
        })
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .filter_map(|r| r.ok())
        .collect();

    let template = MatchExportTemplate {
        white_bot,
        black_bot,
        white_score,
        black_score,
        games,
        created_at,
    };

    let html = template.render().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "text/html; charset=utf-8"),
            (header::CONTENT_DISPOSITION, &format!("attachment; filename=\"match-{}.html\"", match_id)),
        ],
        html,
    ).into_response())
}
```

**Step 5: Register export module and route**

Add to `crates/bot-arena-server/src/api/mod.rs`:

```rust
pub mod export;
```

Add route in the router setup (find where routes are defined):

```rust
.route("/api/export/match/:id", get(export::export_match))
```

**Step 6: Run tests**

Run: `cargo test -p bot-arena-server`
Expected: All tests pass

**Step 7: Commit**

```bash
git add crates/bot-arena-server/templates/export_match.html
git add crates/bot-arena-server/src/templates/match_export.rs
git add crates/bot-arena-server/src/api/export.rs
git add crates/bot-arena-server/src/api/mod.rs
git add crates/bot-arena-server/src/templates/mod.rs
git commit -m "feat(bot-arena-server): add match export HTML endpoint"
```

---

### Task 4: Create game export template and endpoint

**Files:**
- Create: `crates/bot-arena-server/templates/export_game.html`
- Create: `crates/bot-arena-server/src/templates/game_export.rs`
- Modify: `crates/bot-arena-server/src/api/export.rs`
- Modify: `crates/bot-arena-server/src/templates/mod.rs`

**Step 1: Create game export template**

Create `crates/bot-arena-server/templates/export_game.html`:

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Game: {{ white }} vs {{ black }}</title>
    <style>
        :root { --bg: #1a1a2e; --fg: #eee; --accent: #e94560; --border: #0f3460; }
        * { box-sizing: border-box; margin: 0; padding: 0; }
        body { font-family: system-ui, sans-serif; background: var(--bg); color: var(--fg); padding: 2rem; }
        h1 { margin-bottom: 1rem; }
        .game-info { background: var(--border); padding: 1rem; border-radius: 8px; margin-bottom: 2rem; }
        .result { font-size: 1.5rem; font-weight: bold; color: var(--accent); }
        .board-container { display: flex; gap: 2rem; flex-wrap: wrap; }
        .board { flex: 0 0 auto; }
        .moves { flex: 1; min-width: 300px; }
        .move-list { display: grid; grid-template-columns: auto 1fr 1fr; gap: 0.25rem 1rem; }
        .move-number { color: #888; }
        .move { font-family: monospace; }
        footer { margin-top: 2rem; font-size: 0.8rem; color: #888; }
    </style>
</head>
<body>
    <h1>Game Viewer</h1>

    <div class="game-info">
        <p><strong>{{ white }}</strong> (White) vs <strong>{{ black }}</strong> (Black)</p>
        <p class="result">{{ result }}</p>
        {% if let Some(opening) = opening %}
        <p>Opening: {{ opening }}</p>
        {% endif %}
    </div>

    <div class="board-container">
        <div class="board">
            {{ board|safe }}
        </div>

        <div class="moves">
            <h2>Moves</h2>
            <div class="move-list">
                {% for (i, pair) in move_pairs.iter().enumerate() %}
                <span class="move-number">{{ i + 1 }}.</span>
                <span class="move">{{ pair.0 }}</span>
                <span class="move">{% if let Some(black_move) = &pair.1 %}{{ black_move }}{% endif %}</span>
                {% endfor %}
            </div>
        </div>
    </div>

    <footer>
        <p>Generated by Bot Arena</p>
    </footer>
</body>
</html>
```

**Step 2: Create game_export.rs**

Create `crates/bot-arena-server/src/templates/game_export.rs`:

```rust
//! Game export template.

use askama::Template;

/// Game export HTML template.
#[derive(Template)]
#[template(path = "export_game.html")]
pub struct GameExportTemplate {
    pub white: String,
    pub black: String,
    pub result: String,
    pub opening: Option<String>,
    pub board: String,
    pub move_pairs: Vec<(String, Option<String>)>,
}

impl GameExportTemplate {
    /// Convert a flat list of moves into pairs (white, black).
    pub fn pair_moves(moves: Vec<String>) -> Vec<(String, Option<String>)> {
        moves
            .chunks(2)
            .map(|chunk| {
                let white = chunk.first().cloned().unwrap_or_default();
                let black = chunk.get(1).cloned();
                (white, black)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pair_moves_even() {
        let moves = vec!["e4".to_string(), "e5".to_string(), "Nf3".to_string(), "Nc6".to_string()];
        let pairs = GameExportTemplate::pair_moves(moves);
        assert_eq!(pairs.len(), 2);
        assert_eq!(pairs[0], ("e4".to_string(), Some("e5".to_string())));
        assert_eq!(pairs[1], ("Nf3".to_string(), Some("Nc6".to_string())));
    }

    #[test]
    fn test_pair_moves_odd() {
        let moves = vec!["e4".to_string(), "e5".to_string(), "Nf3".to_string()];
        let pairs = GameExportTemplate::pair_moves(moves);
        assert_eq!(pairs.len(), 2);
        assert_eq!(pairs[1], ("Nf3".to_string(), None));
    }

    #[test]
    fn test_game_export_renders() {
        let template = GameExportTemplate {
            white: "minimax".to_string(),
            black: "random".to_string(),
            result: "1-0".to_string(),
            opening: Some("Italian Game".to_string()),
            board: "<svg></svg>".to_string(),
            move_pairs: vec![("e4".to_string(), Some("e5".to_string()))],
        };

        let html = template.render().unwrap();
        assert!(html.contains("minimax"));
        assert!(html.contains("Italian Game"));
    }
}
```

**Step 3: Update templates/mod.rs**

```rust
//! HTML templates for export functionality.

pub mod board;
pub mod game_export;
pub mod match_export;

pub use board::{BoardTemplate, PieceView};
pub use game_export::GameExportTemplate;
pub use match_export::{MatchExportTemplate, GameSummary};
```

**Step 4: Add export_game endpoint to export.rs**

Add to `crates/bot-arena-server/src/api/export.rs`:

```rust
use crate::templates::{BoardTemplate, GameExportTemplate};

/// Export a game as a standalone HTML file.
pub async fn export_game(
    State(state): State<AppState>,
    Path(game_id): Path<String>,
) -> Result<Response, StatusCode> {
    let conn = state.db.lock().unwrap();

    // Get game info
    let game_row: Result<(String, String, String, Option<String>, Option<String>), _> = conn.query_row(
        "SELECT m.white_bot, m.black_bot, g.result, g.game_number, o.name
         FROM games g
         JOIN matches m ON g.match_id = m.id
         LEFT JOIN openings o ON m.opening_id = o.id
         WHERE g.id = ?",
        [&game_id],
        |row| {
            let white: String = row.get(0)?;
            let black: String = row.get(1)?;
            let result: String = row.get(2)?;
            let game_number: Option<i32> = row.get(3)?;
            let opening: Option<String> = row.get(4)?;

            // Swap based on game number
            let (w, b) = if game_number.unwrap_or(0) % 2 == 0 {
                (white, black)
            } else {
                (black, white)
            };

            Ok((w, b, result, opening, None))
        },
    );

    let (white, black, result, opening, _) = game_row.map_err(|_| StatusCode::NOT_FOUND)?;

    // Get moves
    let mut stmt = conn
        .prepare("SELECT san FROM moves WHERE game_id = ? ORDER BY ply")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let moves: Vec<String> = stmt
        .query_map([&game_id], |row| row.get(0))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .filter_map(|r| r.ok())
        .collect();

    // Get final FEN or use starting position
    let final_fen = conn
        .query_row(
            "SELECT fen FROM moves WHERE game_id = ? ORDER BY ply DESC LIMIT 1",
            [&game_id],
            |row| row.get::<_, String>(0),
        )
        .unwrap_or_else(|_| "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string());

    let board = BoardTemplate::from_fen(&final_fen);
    let board_html = board.render().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let template = GameExportTemplate {
        white,
        black,
        result,
        opening,
        board: board_html,
        move_pairs: GameExportTemplate::pair_moves(moves),
    };

    let html = template.render().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "text/html; charset=utf-8"),
            (header::CONTENT_DISPOSITION, &format!("attachment; filename=\"game-{}.html\"", game_id)),
        ],
        html,
    ).into_response())
}
```

**Step 5: Add route**

Add route in router setup:

```rust
.route("/api/export/game/:id", get(export::export_game))
```

**Step 6: Run tests**

Run: `cargo test -p bot-arena-server`
Expected: All tests pass

**Step 7: Commit**

```bash
git add crates/bot-arena-server/templates/export_game.html
git add crates/bot-arena-server/src/templates/game_export.rs
git add crates/bot-arena-server/src/templates/mod.rs
git add crates/bot-arena-server/src/api/export.rs
git commit -m "feat(bot-arena-server): add game export HTML endpoint"
```

---

### Task 5: Create bot profile export template and endpoint

**Files:**
- Create: `crates/bot-arena-server/templates/export_bot.html`
- Create: `crates/bot-arena-server/src/templates/bot_export.rs`
- Modify: `crates/bot-arena-server/src/api/export.rs`
- Modify: `crates/bot-arena-server/src/templates/mod.rs`

**Step 1: Create bot export template**

Create `crates/bot-arena-server/templates/export_bot.html`:

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Bot Profile: {{ name }}</title>
    <style>
        :root { --bg: #1a1a2e; --fg: #eee; --accent: #e94560; --border: #0f3460; }
        * { box-sizing: border-box; margin: 0; padding: 0; }
        body { font-family: system-ui, sans-serif; background: var(--bg); color: var(--fg); padding: 2rem; }
        h1 { margin-bottom: 1rem; }
        .profile { background: var(--border); padding: 1rem; border-radius: 8px; margin-bottom: 2rem; }
        .elo { font-size: 2.5rem; font-weight: bold; color: var(--accent); }
        .stats { display: grid; grid-template-columns: repeat(auto-fit, minmax(150px, 1fr)); gap: 1rem; margin-top: 1rem; }
        .stat { text-align: center; }
        .stat-value { font-size: 1.5rem; font-weight: bold; }
        .stat-label { color: #888; font-size: 0.9rem; }
        .elo-chart { margin: 2rem 0; }
        .elo-chart svg { width: 100%; max-width: 600px; height: 200px; }
        footer { margin-top: 2rem; font-size: 0.8rem; color: #888; }
    </style>
</head>
<body>
    <h1>{{ name }}</h1>

    <div class="profile">
        <p class="elo">{{ elo }} Elo</p>

        <div class="stats">
            <div class="stat">
                <div class="stat-value">{{ games_played }}</div>
                <div class="stat-label">Games</div>
            </div>
            <div class="stat">
                <div class="stat-value">{{ wins }}</div>
                <div class="stat-label">Wins</div>
            </div>
            <div class="stat">
                <div class="stat-value">{{ draws }}</div>
                <div class="stat-label">Draws</div>
            </div>
            <div class="stat">
                <div class="stat-value">{{ losses }}</div>
                <div class="stat-label">Losses</div>
            </div>
            <div class="stat">
                <div class="stat-value">{{ win_rate }}%</div>
                <div class="stat-label">Win Rate</div>
            </div>
        </div>
    </div>

    {% if !elo_history.is_empty() %}
    <div class="elo-chart">
        <h2>Elo History</h2>
        {{ elo_chart|safe }}
    </div>
    {% endif %}

    <footer>
        <p>Generated by Bot Arena</p>
    </footer>
</body>
</html>
```

**Step 2: Create bot_export.rs**

Create `crates/bot-arena-server/src/templates/bot_export.rs`:

```rust
//! Bot profile export template.

use askama::Template;

/// Elo history point.
#[derive(Debug, Clone)]
pub struct EloPoint {
    pub elo: i32,
    pub date: String,
}

/// Bot profile export HTML template.
#[derive(Template)]
#[template(path = "export_bot.html")]
pub struct BotExportTemplate {
    pub name: String,
    pub elo: i32,
    pub games_played: i32,
    pub wins: i32,
    pub draws: i32,
    pub losses: i32,
    pub win_rate: String,
    pub elo_history: Vec<EloPoint>,
    pub elo_chart: String,
}

impl BotExportTemplate {
    /// Generate a simple SVG line chart for Elo history.
    pub fn generate_elo_chart(history: &[EloPoint]) -> String {
        if history.is_empty() {
            return String::new();
        }

        let min_elo = history.iter().map(|p| p.elo).min().unwrap_or(1000) - 50;
        let max_elo = history.iter().map(|p| p.elo).max().unwrap_or(2000) + 50;
        let range = (max_elo - min_elo) as f64;

        let width = 600.0;
        let height = 200.0;
        let padding = 40.0;

        let points: Vec<String> = history
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let x = padding + (i as f64 / (history.len() - 1).max(1) as f64) * (width - 2.0 * padding);
                let y = height - padding - ((p.elo - min_elo) as f64 / range) * (height - 2.0 * padding);
                format!("{:.1},{:.1}", x, y)
            })
            .collect();

        format!(
            r#"<svg viewBox="0 0 {} {}" xmlns="http://www.w3.org/2000/svg">
                <rect fill="#0f3460" width="{}" height="{}"/>
                <polyline fill="none" stroke="#e94560" stroke-width="2" points="{}"/>
                <text x="{}" y="15" fill="#888" font-size="12">{}</text>
                <text x="{}" y="{}" fill="#888" font-size="12">{}</text>
            </svg>"#,
            width, height,
            width, height,
            points.join(" "),
            padding, max_elo,
            padding, height - 5.0, min_elo
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_elo_chart() {
        let history = vec![
            EloPoint { elo: 1500, date: "2025-01-01".to_string() },
            EloPoint { elo: 1520, date: "2025-01-02".to_string() },
            EloPoint { elo: 1510, date: "2025-01-03".to_string() },
        ];

        let chart = BotExportTemplate::generate_elo_chart(&history);
        assert!(chart.contains("<svg"));
        assert!(chart.contains("polyline"));
    }

    #[test]
    fn test_bot_export_renders() {
        let template = BotExportTemplate {
            name: "minimax".to_string(),
            elo: 1650,
            games_played: 100,
            wins: 60,
            draws: 20,
            losses: 20,
            win_rate: "70.0".to_string(),
            elo_history: vec![],
            elo_chart: String::new(),
        };

        let html = template.render().unwrap();
        assert!(html.contains("minimax"));
        assert!(html.contains("1650 Elo"));
    }
}
```

**Step 3: Update templates/mod.rs**

```rust
//! HTML templates for export functionality.

pub mod board;
pub mod bot_export;
pub mod game_export;
pub mod match_export;

pub use board::{BoardTemplate, PieceView};
pub use bot_export::{BotExportTemplate, EloPoint};
pub use game_export::GameExportTemplate;
pub use match_export::{MatchExportTemplate, GameSummary};
```

**Step 4: Add export_bot endpoint to export.rs**

Add to `crates/bot-arena-server/src/api/export.rs`:

```rust
use crate::templates::{BotExportTemplate, EloPoint};

/// Export a bot profile as a standalone HTML file.
pub async fn export_bot(
    State(state): State<AppState>,
    Path(bot_name): Path<String>,
) -> Result<Response, StatusCode> {
    let conn = state.db.lock().unwrap();

    // Get bot info
    let bot_row: Result<(i32,), _> = conn.query_row(
        "SELECT elo FROM bots WHERE name = ?",
        [&bot_name],
        |row| Ok((row.get(0)?,)),
    );

    let (elo,) = bot_row.map_err(|_| StatusCode::NOT_FOUND)?;

    // Get stats
    let stats: (i32, i32, i32, i32) = conn
        .query_row(
            "SELECT
                COUNT(*) as games,
                SUM(CASE WHEN (white_bot = ?1 AND result = '1-0') OR (black_bot = ?1 AND result = '0-1') THEN 1 ELSE 0 END) as wins,
                SUM(CASE WHEN result = '1/2-1/2' THEN 1 ELSE 0 END) as draws,
                SUM(CASE WHEN (white_bot = ?1 AND result = '0-1') OR (black_bot = ?1 AND result = '1-0') THEN 1 ELSE 0 END) as losses
             FROM games g
             JOIN matches m ON g.match_id = m.id
             WHERE m.white_bot = ?1 OR m.black_bot = ?1",
            [&bot_name],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .unwrap_or((0, 0, 0, 0));

    let (games_played, wins, draws, losses) = stats;
    let win_rate = if games_played > 0 {
        format!("{:.1}", (wins as f64 + draws as f64 * 0.5) / games_played as f64 * 100.0)
    } else {
        "0.0".to_string()
    };

    // Get Elo history
    let mut stmt = conn
        .prepare("SELECT elo_rating, recorded_at FROM elo_history WHERE bot_name = ? ORDER BY recorded_at")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let elo_history: Vec<EloPoint> = stmt
        .query_map([&bot_name], |row| {
            Ok(EloPoint {
                elo: row.get(0)?,
                date: row.get::<_, String>(1).unwrap_or_default(),
            })
        })
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .filter_map(|r| r.ok())
        .collect();

    let elo_chart = BotExportTemplate::generate_elo_chart(&elo_history);

    let template = BotExportTemplate {
        name: bot_name.clone(),
        elo,
        games_played,
        wins,
        draws,
        losses,
        win_rate,
        elo_history,
        elo_chart,
    };

    let html = template.render().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "text/html; charset=utf-8"),
            (header::CONTENT_DISPOSITION, &format!("attachment; filename=\"bot-{}.html\"", bot_name)),
        ],
        html,
    ).into_response())
}
```

**Step 5: Add route**

```rust
.route("/api/export/bot/:name", get(export::export_bot))
```

**Step 6: Run tests**

Run: `cargo test -p bot-arena-server`
Expected: All tests pass

**Step 7: Commit**

```bash
git add crates/bot-arena-server/templates/export_bot.html
git add crates/bot-arena-server/src/templates/bot_export.rs
git add crates/bot-arena-server/src/templates/mod.rs
git add crates/bot-arena-server/src/api/export.rs
git commit -m "feat(bot-arena-server): add bot profile export HTML endpoint"
```

---

### Task 6: Add export buttons to UI pages

**Files:**
- Modify: `apps/web/bot-arena-ui/src/lib/api.ts`
- Modify: `apps/web/bot-arena-ui/src/routes/games/[id]/+page.svelte`
- Modify: `apps/web/bot-arena-ui/src/routes/bots/[name]/+page.svelte`

**Step 1: Add export helper to api.ts**

Add to `apps/web/bot-arena-ui/src/lib/api.ts`:

```typescript
export function getExportUrl(type: 'match' | 'game' | 'bot', id: string): string {
  return `${API_BASE}/export/${type}/${encodeURIComponent(id)}`;
}
```

**Step 2: Add export button to game detail page**

Find the game info section in `apps/web/bot-arena-ui/src/routes/games/[id]/+page.svelte` and add:

```svelte
<a href={getExportUrl('game', gameId)} download class="export-btn">
  Export HTML
</a>
```

Add import:
```svelte
import { getExportUrl } from '$lib/api';
```

Add CSS:
```css
.export-btn {
  display: inline-block;
  padding: 0.5rem 1rem;
  background: var(--border);
  color: var(--fg);
  text-decoration: none;
  border-radius: 4px;
  font-size: 0.9rem;
}

.export-btn:hover {
  background: var(--highlight);
}
```

**Step 3: Add export button to bot profile page**

Find the profile section in `apps/web/bot-arena-ui/src/routes/bots/[name]/+page.svelte` and add:

```svelte
<a href={getExportUrl('bot', botName)} download class="export-btn">
  Export Profile
</a>
```

Add import:
```svelte
import { getExportUrl } from '$lib/api';
```

**Step 4: Build and verify**

Run: `cd apps/web/bot-arena-ui && pnpm build`
Expected: Build succeeds

**Step 5: Commit**

```bash
git add apps/web/bot-arena-ui/src/lib/api.ts
git add apps/web/bot-arena-ui/src/routes/games/\[id\]/+page.svelte
git add apps/web/bot-arena-ui/src/routes/bots/\[name\]/+page.svelte
git commit -m "feat(bot-arena-ui): add export buttons to game and bot pages"
```

---

## Track B: UI Preset Integration

### Task 7: Add description field to PresetConfig

**Files:**
- Modify: `crates/bot-arena/src/config.rs`
- Modify: `arena.toml`

**Step 1: Add description field to PresetConfig**

In `crates/bot-arena/src/config.rs`, update the `PresetConfig` struct:

```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PresetConfig {
    /// Number of games to play.
    #[serde(default = "default_games")]
    pub games: u32,

    /// Time control string for UCI.
    #[serde(default = "default_time_control")]
    pub time_control: String,

    /// Opening names to use.
    #[serde(default)]
    pub openings: Vec<String>,

    /// Human-readable description of the preset.
    #[serde(default)]
    pub description: String,
}
```

**Step 2: Update arena.toml with descriptions**

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

**Step 3: Run tests**

Run: `cargo test -p bot-arena config`
Expected: All tests pass

**Step 4: Commit**

```bash
git add crates/bot-arena/src/config.rs arena.toml
git commit -m "feat(bot-arena): add description field to presets"
```

---

### Task 8: Create GET /api/presets endpoint

**Files:**
- Create: `crates/bot-arena-server/src/api/presets.rs`
- Modify: `crates/bot-arena-server/src/api/mod.rs`
- Modify: router setup

**Step 1: Create presets.rs**

Create `crates/bot-arena-server/src/api/presets.rs`:

```rust
//! Preset configuration API.

use axum::{extract::State, Json};
use serde::Serialize;

use crate::AppState;

/// A preset returned by the API.
#[derive(Debug, Serialize)]
pub struct PresetResponse {
    pub name: String,
    pub games: u32,
    pub time_control: String,
    pub description: String,
}

/// List all available presets.
pub async fn list_presets(State(state): State<AppState>) -> Json<Vec<PresetResponse>> {
    let presets: Vec<PresetResponse> = state
        .config
        .presets
        .iter()
        .map(|(name, preset)| PresetResponse {
            name: name.clone(),
            games: preset.games,
            time_control: preset.time_control.clone(),
            description: preset.description.clone(),
        })
        .collect();

    Json(presets)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preset_response_serializes() {
        let preset = PresetResponse {
            name: "quick".to_string(),
            games: 10,
            time_control: "movetime 100".to_string(),
            description: "Fast test".to_string(),
        };

        let json = serde_json::to_string(&preset).unwrap();
        assert!(json.contains("quick"));
        assert!(json.contains("Fast test"));
    }
}
```

**Step 2: Add AppState config field if not present**

Ensure `AppState` has access to the config. If not already present, add:

```rust
pub struct AppState {
    // ... existing fields ...
    pub config: Arc<ArenaConfig>,
}
```

**Step 3: Register module and route**

Add to `crates/bot-arena-server/src/api/mod.rs`:

```rust
pub mod presets;
```

Add route:

```rust
.route("/api/presets", get(presets::list_presets))
```

**Step 4: Run tests**

Run: `cargo test -p bot-arena-server presets`
Expected: All tests pass

**Step 5: Commit**

```bash
git add crates/bot-arena-server/src/api/presets.rs
git add crates/bot-arena-server/src/api/mod.rs
git commit -m "feat(bot-arena-server): add presets API endpoint"
```

---

### Task 9: Add preset dropdown to match creation form

**Files:**
- Modify: `apps/web/bot-arena-ui/src/lib/api.ts`
- Modify: `apps/web/bot-arena-ui/src/routes/match/new/+page.svelte`

**Step 1: Add preset types and fetch function to api.ts**

Add to `apps/web/bot-arena-ui/src/lib/api.ts`:

```typescript
export interface Preset {
  name: string;
  games: number;
  time_control: string;
  description: string;
}

export async function getPresets(): Promise<Preset[]> {
  const response = await fetch(`${API_BASE}/presets`);
  if (!response.ok) {
    throw new Error('Failed to fetch presets');
  }
  return response.json();
}
```

**Step 2: Add preset dropdown to match creation form**

In `apps/web/bot-arena-ui/src/routes/match/new/+page.svelte`:

Add to script section:
```svelte
import { getPresets, type Preset } from '$lib/api';

let presets = $state<Preset[]>([]);
let selectedPreset = $state('custom');

$effect(() => {
  getPresets().then(p => { presets = p; });
});

function applyPreset(presetName: string) {
  if (presetName === 'custom') return;

  const preset = presets.find(p => p.name === presetName);
  if (preset) {
    games = preset.games;
    // Parse movetime from time_control string
    const match = preset.time_control.match(/movetime\s+(\d+)/);
    if (match) {
      movetime = parseInt(match[1], 10);
    }
  }
}
```

Add to form, before the games input:
```svelte
<div class="form-group">
  <label for="preset">Preset</label>
  <select
    id="preset"
    bind:value={selectedPreset}
    onchange={() => applyPreset(selectedPreset)}
  >
    <option value="custom">Custom</option>
    {#each presets as preset}
      <option value={preset.name}>{preset.name} - {preset.description}</option>
    {/each}
  </select>
</div>
```

**Step 3: Build and verify**

Run: `cd apps/web/bot-arena-ui && pnpm build`
Expected: Build succeeds

**Step 4: Commit**

```bash
git add apps/web/bot-arena-ui/src/lib/api.ts
git add apps/web/bot-arena-ui/src/routes/match/new/+page.svelte
git commit -m "feat(bot-arena-ui): add preset dropdown to match creation"
```

---

### Task 10: Auto-fill form fields on preset selection

**Files:**
- Modify: `apps/web/bot-arena-ui/src/routes/match/new/+page.svelte`

**Step 1: Add visual feedback when preset is applied**

Update the form to show which preset is active:

```svelte
<div class="form-group">
  <label for="games">
    Games
    {#if selectedPreset !== 'custom'}
      <span class="preset-badge">from {selectedPreset}</span>
    {/if}
  </label>
  <input type="number" id="games" bind:value={games} min="1" />
</div>

<div class="form-group">
  <label for="movetime">
    Move Time (ms)
    {#if selectedPreset !== 'custom'}
      <span class="preset-badge">from {selectedPreset}</span>
    {/if}
  </label>
  <input type="number" id="movetime" bind:value={movetime} min="100" step="100" />
</div>
```

**Step 2: Add CSS for preset badge**

```css
.preset-badge {
  font-size: 0.75rem;
  background: var(--highlight);
  color: var(--bg);
  padding: 0.1rem 0.4rem;
  border-radius: 4px;
  margin-left: 0.5rem;
}
```

**Step 3: Reset to custom when manually editing**

```svelte
<input
  type="number"
  id="games"
  bind:value={games}
  min="1"
  oninput={() => { selectedPreset = 'custom'; }}
/>
```

Apply same pattern to movetime input.

**Step 4: Build and verify**

Run: `cd apps/web/bot-arena-ui && pnpm build`
Expected: Build succeeds

**Step 5: Commit**

```bash
git add apps/web/bot-arena-ui/src/routes/match/new/+page.svelte
git commit -m "feat(bot-arena-ui): add preset auto-fill with visual feedback"
```

---

## Track C: Documentation & Cleanup

### Task 11: Fix doc warnings in chess-analysis and chess-openings

**Files:**
- Modify: `crates/chess-analysis/src/*.rs` (files with `<value>` in docs)
- Modify: `crates/chess-openings/src/*.rs` (file with `find_by_moves` link)

**Step 1: Find and fix chess-analysis warnings**

Run: `cargo doc -p chess-analysis 2>&1 | grep "unclosed"`

The warning is about `<value>` being interpreted as HTML. Fix by escaping:

Change `<value>` to `` `<value>` `` in doc comments.

**Step 2: Find and fix chess-openings warning**

Run: `cargo doc -p chess-openings 2>&1 | grep "unresolved"`

Fix the link to `find_by_moves` - either use full path `[`find_by_moves`](Self::find_by_moves)` or remove the link.

**Step 3: Verify doc build is clean**

Run: `cargo doc --workspace --no-deps 2>&1 | grep -E "warning|error" | wc -l`
Expected: 0

**Step 4: Commit**

```bash
git add crates/chess-analysis crates/chess-openings
git commit -m "docs: fix documentation warnings in chess-analysis and chess-openings"
```

---

### Task 12: Remove obsolete TODO, implement ECO lookup

**Files:**
- Modify: `crates/bot-arena-server/src/api/matches.rs`
- Modify: `crates/bot-arena-server/src/api/openings.rs`

**Step 1: Remove obsolete TODO in matches.rs**

Find and remove the comment:
```rust
// TODO: Trigger match runner (Phase G)
```

This is obsolete - the worker handles match execution.

**Step 2: Implement ECO lookup in openings.rs**

Find the TODO:
```rust
eco: String::new(), // TODO: lookup ECO code from opening database
```

Replace with actual ECO lookup using the chess-openings crate:

```rust
// Add import at top
use chess_openings::OpeningDatabase;

// In the handler, load database and lookup ECO
let opening_db = OpeningDatabase::load().unwrap_or_default();

// When mapping openings, lookup ECO:
let eco = opening_db
    .find_by_name(&opening_name)
    .and_then(|o| o.eco.clone())
    .unwrap_or_default();
```

**Step 3: Run tests**

Run: `cargo test -p bot-arena-server`
Expected: All tests pass

**Step 4: Commit**

```bash
git add crates/bot-arena-server/src/api/matches.rs
git add crates/bot-arena-server/src/api/openings.rs
git commit -m "fix(bot-arena-server): remove obsolete TODO, implement ECO lookup"
```

---

### Task 13: Add fail_match() to worker db module

**Files:**
- Modify: `crates/bot-arena-worker/src/db.rs`
- Modify: `crates/bot-arena-worker/src/main.rs`

**Step 1: Add fail_match function to db.rs**

Add to `crates/bot-arena-worker/src/db.rs`:

```rust
/// Mark a match as failed with an error message.
pub fn fail_match(conn: &Connection, match_id: &str, error: &str) -> Result<()> {
    conn.execute(
        "UPDATE matches SET status = 'failed', error_message = ? WHERE id = ?",
        params![error, match_id],
    )?;
    Ok(())
}
```

**Step 2: Add error_message column if not present**

Check if the column exists, if not add migration:

```sql
ALTER TABLE matches ADD COLUMN error_message TEXT;
```

**Step 3: Update main.rs to use fail_match**

Find the TODO:
```rust
// TODO: Mark match as failed in database
```

Replace with:
```rust
if let Err(e) = db::fail_match(&db, &pending.id, &e.to_string()) {
    tracing::error!("Failed to mark match as failed: {}", e);
}
```

**Step 4: Add test for fail_match**

```rust
#[test]
fn test_fail_match() {
    let conn = Connection::open_in_memory().unwrap();
    // Setup schema...
    // Insert test match...

    fail_match(&conn, "test-match", "Engine crashed").unwrap();

    let status: String = conn.query_row(
        "SELECT status FROM matches WHERE id = ?",
        ["test-match"],
        |row| row.get(0),
    ).unwrap();

    assert_eq!(status, "failed");
}
```

**Step 5: Run tests**

Run: `cargo test -p bot-arena-worker`
Expected: All tests pass

**Step 6: Commit**

```bash
git add crates/bot-arena-worker/src/db.rs
git add crates/bot-arena-worker/src/main.rs
git commit -m "feat(bot-arena-worker): add fail_match to mark failed matches in database"
```

---

### Task 14: Add README.md to key crates

**Files:**
- Create: `crates/bot-arena-server/README.md`
- Create: `crates/bot-arena-worker/README.md`
- Create: `crates/chess-analysis/README.md`

**Step 1: Create bot-arena-server README**

Create `crates/bot-arena-server/README.md`:

```markdown
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
| `/api/export/*` | GET | Export HTML reports |
| `/ws` | WS | Real-time updates |

## Configuration

See `arena.toml` in project root.
```

**Step 2: Create bot-arena-worker README**

Create `crates/bot-arena-worker/README.md`:

```markdown
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
```

**Step 3: Create chess-analysis README**

Create `crates/chess-analysis/README.md`:

```markdown
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
```

**Step 4: Commit**

```bash
git add crates/bot-arena-server/README.md
git add crates/bot-arena-worker/README.md
git add crates/chess-analysis/README.md
git commit -m "docs: add README files to key crates"
```

---

### Task 15: Create docs/architecture.md

**Files:**
- Create: `docs/architecture.md`

**Step 1: Create architecture.md**

Create `docs/architecture.md`:

```markdown
# Bot Arena Architecture

## System Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Bot Arena System                             │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌──────────────┐     ┌──────────────┐     ┌────────────────────┐   │
│  │  Svelte UI   │────►│   Axum       │────►│   SQLite Database  │   │
│  │  (Browser)   │◄────│   Server     │◄────│                    │   │
│  └──────────────┘     └──────────────┘     └─────────┬──────────┘   │
│         │                    │                       │              │
│         │ WebSocket          │ Stockfish             │ Poll/Write   │
│         ▼                    ▼                       ▼              │
│  ┌──────────────┐     ┌──────────────┐     ┌────────────────────┐   │
│  │  Live Game   │     │   Engine     │     │   Worker Process   │   │
│  │  Updates     │     │   Pool       │     │   (UCI Execution)  │   │
│  └──────────────┘     └──────────────┘     └────────────────────┘   │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## Components

### Frontend (`apps/web/bot-arena-ui`)

- **Framework**: SvelteKit 2 with Svelte 5 runes
- **Styling**: CSS variables for theming
- **State**: Svelte 5 `$state()` and `$derived()`
- **API**: Fetch-based client in `src/lib/api.ts`

### Server (`crates/bot-arena-server`)

- **Framework**: Axum with Tower middleware
- **Database**: SQLite via rusqlite
- **WebSocket**: Real-time game move broadcasting
- **Analysis**: Stockfish process pool for position evaluation

### Worker (`crates/bot-arena-worker`)

- **Role**: Execute matches by spawning UCI engine processes
- **Scaling**: Multiple workers can run in parallel
- **Communication**: Shared SQLite database (no direct server communication)

### Shared Libraries

| Crate | Purpose |
|-------|---------|
| `chess-engine` | Core chess logic, move generation |
| `chess-openings` | Opening database, ECO codes |
| `chess-analysis` | Move quality analysis |
| `bot-arena` | CLI for running matches |
| `uci` | UCI protocol implementation |

## Data Flow

### Match Creation

1. User submits match form in UI
2. Server creates match record with `status = 'pending'`
3. Returns match ID to UI

### Match Execution

1. Worker polls for `status = 'pending'`
2. Claims match with atomic update (`status = 'running'`)
3. Spawns UCI engine processes
4. Plays games, writing moves to database
5. Server broadcasts moves via WebSocket
6. Worker updates final scores and Elo
7. Sets `status = 'completed'`

### Position Analysis

1. User requests analysis in game viewer
2. Server assigns request to engine pool
3. Stockfish evaluates position
4. Returns score, best move, principal variation

## Database Schema

Key tables:
- `bots` - Bot definitions with Elo ratings
- `matches` - Match metadata and scores
- `games` - Individual game records
- `moves` - Move-by-move game data
- `elo_history` - Elo rating changes over time
- `openings` - Opening database

## Configuration

All configuration in `arena.toml`:
- Bot executable paths
- Match presets
- Time controls
- Server/worker settings
```

**Step 2: Commit**

```bash
git add docs/architecture.md
git commit -m "docs: add architecture overview"
```

---

### Task 16: Create docs/deployment.md

**Files:**
- Create: `docs/deployment.md`

**Step 1: Create deployment.md**

Create `docs/deployment.md`:

```markdown
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

Binaries will be in `target/release/`.

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
path = "./bots/minimax"
time_control = "movetime 500"

[bots.random]
path = "./bots/random"

[presets.quick]
games = 10
time_control = "movetime 100"
description = "Fast test run"
```

## Running

### Development

```bash
# Terminal 1: Server
cargo run -p bot-arena-server -- --db data/arena.db --static-dir apps/web/bot-arena-ui/build

# Terminal 2: Worker
cargo run -p bot-arena-worker -- --db data/arena.db --bots-dir ./bots
```

### Production

```bash
# Server
./target/release/bot-arena-server \
  --db /var/lib/bot-arena/arena.db \
  --static-dir /var/www/bot-arena \
  --port 8080

# Worker(s)
./target/release/bot-arena-worker \
  --db /var/lib/bot-arena/arena.db \
  --bots-dir /opt/bots
```

## Stockfish Setup

For position analysis, install Stockfish:

```bash
# macOS
brew install stockfish

# Ubuntu/Debian
apt install stockfish

# Or download from https://stockfishchess.org/download/
```

The server will auto-detect Stockfish in PATH.

## Scaling Workers

Multiple workers can run simultaneously:

```bash
# Run 4 workers
for i in {1..4}; do
  ./bot-arena-worker --db arena.db --bots-dir ./bots &
done
```

Each worker claims different matches via atomic database updates.

## Monitoring

Check server logs for:
- API request timing
- WebSocket connections
- Engine pool utilization

Check worker logs for:
- Match claiming
- Game progress
- Engine errors

## Troubleshooting

### "Database is locked"
- Ensure only one writer at a time
- Consider WAL mode: `PRAGMA journal_mode=WAL;`

### "Engine not found"
- Verify bot path in arena.toml
- Check executable permissions

### "Analysis timeout"
- Reduce analysis depth
- Increase pool size
- Check Stockfish installation
```

**Step 2: Commit**

```bash
git add docs/deployment.md
git commit -m "docs: add deployment guide"
```

---

## Track D: Performance Optimization

### Task 17: Add database indexes for common queries

**Files:**
- Modify: `crates/bot-arena-server/src/db.rs`

**Step 1: Add indexes in schema initialization**

Find the schema setup in `crates/bot-arena-server/src/db.rs` and add:

```rust
// Add after table creation
conn.execute_batch(
    "CREATE INDEX IF NOT EXISTS idx_games_match_id ON games(match_id);
     CREATE INDEX IF NOT EXISTS idx_moves_game_id ON moves(game_id);
     CREATE INDEX IF NOT EXISTS idx_matches_status ON matches(status);
     CREATE INDEX IF NOT EXISTS idx_elo_history_bot_name ON elo_history(bot_name);
     CREATE INDEX IF NOT EXISTS idx_matches_white_bot ON matches(white_bot);
     CREATE INDEX IF NOT EXISTS idx_matches_black_bot ON matches(black_bot);"
)?;
```

**Step 2: Verify indexes are created**

Run: `sqlite3 data/arena.db ".indexes"`
Expected: Shows all new indexes

**Step 3: Test query performance**

```bash
sqlite3 data/arena.db "EXPLAIN QUERY PLAN SELECT * FROM games WHERE match_id = 'test'"
```
Expected: Shows "USING INDEX idx_games_match_id"

**Step 4: Commit**

```bash
git add crates/bot-arena-server/src/db.rs
git commit -m "perf(bot-arena-server): add database indexes for common queries"
```

---

### Task 18: Add timing logs to API endpoints

**Files:**
- Create: `crates/bot-arena-server/src/middleware/timing.rs`
- Modify: `crates/bot-arena-server/src/lib.rs`
- Modify: router setup

**Step 1: Create timing middleware**

Create `crates/bot-arena-server/src/middleware/timing.rs`:

```rust
//! Request timing middleware.

use axum::{
    body::Body,
    http::Request,
    middleware::Next,
    response::Response,
};
use std::time::Instant;

pub async fn timing_middleware(request: Request<Body>, next: Next) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let start = Instant::now();

    let response = next.run(request).await;

    let duration = start.elapsed();
    let status = response.status();

    // Log slow requests (>100ms)
    if duration.as_millis() > 100 {
        tracing::warn!(
            method = %method,
            uri = %uri,
            status = %status,
            duration_ms = duration.as_millis(),
            "Slow request"
        );
    } else {
        tracing::debug!(
            method = %method,
            uri = %uri,
            status = %status,
            duration_ms = duration.as_millis(),
            "Request completed"
        );
    }

    response
}
```

**Step 2: Create middleware module**

Create `crates/bot-arena-server/src/middleware/mod.rs`:

```rust
pub mod timing;

pub use timing::timing_middleware;
```

**Step 3: Add middleware module to lib.rs**

```rust
pub mod middleware;
```

**Step 4: Apply middleware to router**

In router setup:

```rust
use axum::middleware;
use crate::middleware::timing_middleware;

let app = Router::new()
    // ... routes ...
    .layer(middleware::from_fn(timing_middleware));
```

**Step 5: Run server and verify logs**

Run server, make requests, check for timing logs.

**Step 6: Commit**

```bash
git add crates/bot-arena-server/src/middleware/
git add crates/bot-arena-server/src/lib.rs
git commit -m "perf(bot-arena-server): add request timing middleware"
```

---

### Task 19: Make engine pool size configurable, add lazy init

**Files:**
- Modify: `crates/bot-arena-server/src/analysis.rs` (or wherever engine pool is defined)
- Modify: config loading

**Step 1: Add pool_size to config**

Add to `arena.toml`:

```toml
[analysis]
pool_size = 2
stockfish_path = "stockfish"
```

**Step 2: Update engine pool to use config**

Find the engine pool initialization and update to:

```rust
pub struct EnginePoolConfig {
    pub pool_size: usize,
    pub stockfish_path: String,
}

impl Default for EnginePoolConfig {
    fn default() -> Self {
        Self {
            pool_size: 2,
            stockfish_path: "stockfish".to_string(),
        }
    }
}

pub struct EnginePool {
    config: EnginePoolConfig,
    engines: Mutex<Vec<Option<StockfishEngine>>>,
}

impl EnginePool {
    pub fn new(config: EnginePoolConfig) -> Self {
        // Lazy init - don't spawn engines until needed
        let engines = (0..config.pool_size).map(|_| None).collect();
        Self {
            config,
            engines: Mutex::new(engines),
        }
    }

    pub async fn acquire(&self) -> Result<StockfishEngine, Error> {
        let mut engines = self.engines.lock().await;

        // Find or create an available engine
        for slot in engines.iter_mut() {
            if let Some(engine) = slot.take() {
                return Ok(engine);
            }
        }

        // All slots occupied, spawn new if under limit
        // Or wait for one to become available
        // ...
    }
}
```

**Step 3: Test lazy initialization**

Verify that engines are not spawned until first analysis request.

**Step 4: Commit**

```bash
git add crates/bot-arena-server/src/analysis.rs
git add arena.toml
git commit -m "perf(bot-arena-server): make engine pool configurable with lazy init"
```

---

### Task 20: Add Cache-Control headers for appropriate endpoints

**Files:**
- Modify: `crates/bot-arena-server/src/api/bots.rs`
- Modify: `crates/bot-arena-server/src/api/openings.rs`
- Modify: `crates/bot-arena-server/src/api/matches.rs`

**Step 1: Add cache headers to openings endpoint**

Openings list is static - can cache for a long time:

```rust
use axum::http::header;

pub async fn list_openings(...) -> impl IntoResponse {
    // ... existing logic ...

    (
        StatusCode::OK,
        [(header::CACHE_CONTROL, "public, max-age=86400")], // 24 hours
        Json(openings),
    )
}
```

**Step 2: Add cache headers to completed match details**

Completed matches are immutable:

```rust
pub async fn get_match(...) -> impl IntoResponse {
    // ... fetch match ...

    let cache_header = if match_data.status == "completed" {
        "public, max-age=31536000" // 1 year for completed
    } else {
        "no-cache" // Always revalidate for in-progress
    };

    (
        StatusCode::OK,
        [(header::CACHE_CONTROL, cache_header)],
        Json(match_data),
    )
}
```

**Step 3: Add short cache to bot stats**

Bot stats change after matches, short cache:

```rust
pub async fn get_bot(...) -> impl IntoResponse {
    // ... existing logic ...

    (
        StatusCode::OK,
        [(header::CACHE_CONTROL, "public, max-age=60")], // 1 minute
        Json(bot),
    )
}
```

**Step 4: Verify headers**

```bash
curl -I http://localhost:3000/api/openings | grep -i cache
```
Expected: `Cache-Control: public, max-age=86400`

**Step 5: Commit**

```bash
git add crates/bot-arena-server/src/api/
git commit -m "perf(bot-arena-server): add Cache-Control headers to API responses"
```

---

## Final Integration Test

### Task 21: Verify all features work together

**Step 1: Build everything**

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
```

**Step 2: Build frontend**

```bash
cd apps/web/bot-arena-ui
pnpm check
pnpm build
```

**Step 3: Manual testing checklist**

- [ ] Server starts without errors
- [ ] Worker connects and processes matches
- [ ] Preset dropdown works in match creation
- [ ] Export buttons download HTML files
- [ ] Exported HTML displays correctly standalone
- [ ] API timing logs appear for requests
- [ ] Cache headers present on responses

**Step 4: Final commit**

```bash
git add -A
git commit -m "test: verify Phase 7 integration"
```

---

**Total: 21 tasks**

Ready for execution.
