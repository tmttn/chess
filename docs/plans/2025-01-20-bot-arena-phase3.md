# Bot Arena Phase 3: Opening Database Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Create a chess openings database crate with built-in openings, ECO import, and search/filter capabilities for use in bot-arena matches.

**Architecture:** Create a `chess-openings` crate that stores openings with metadata (name, ECO code, moves, tags). Include ~50 built-in popular openings and provide search/filter APIs. Integrate with bot-arena for opening selection in matches.

**Tech Stack:** Rust, Serde for serialization

---

## Task 1: Create chess-openings crate scaffold

**Files:**
- Create: `crates/chess-openings/Cargo.toml`
- Create: `crates/chess-openings/src/lib.rs`
- Modify: `Cargo.toml` (workspace)

**Requirements:**
1. Add crate to workspace members
2. Create Cargo.toml with serde, serde_json, thiserror, rand dependencies
3. Create lib.rs with module structure (opening, database, builtin)
4. Create placeholder module files
5. Verify build and commit

---

## Task 2: Add Opening data structures

**Files:**
- Create: `crates/chess-openings/src/opening.rs`

**Requirements:**
1. `OpeningSource` enum: BuiltIn, Eco, Lichess, Custom
2. `OpeningStats` struct: games_played, white_wins, draws, black_wins
3. `Opening` struct with: id, name, eco, moves, fen, source, tags, stats
4. Builder pattern methods: with_eco(), with_source(), with_tags(), with_stats()
5. Helper methods: ply_count(), move_count()
6. Tests for all functionality
7. Serde serialization support

---

## Task 3: Add OpeningDatabase with search

**Files:**
- Create: `crates/chess-openings/src/database.rs`

**Requirements:**
1. `DatabaseError` enum for errors
2. `OpeningDatabase` struct with methods:
   - new(), with_openings(), len(), is_empty(), add()
   - all(), by_id(), by_eco(), by_tag(), by_source()
   - search() - case-insensitive name search
   - popular(n) - top N by games played
   - random_subset(n) - N random openings
   - weighted_random(n) - weighted by popularity
   - filter() - generic predicate filter
3. Comprehensive tests

---

## Task 4: Add built-in openings (~50)

**Files:**
- Create: `crates/chess-openings/src/builtin.rs`

**Requirements:**
1. builtin_openings() function returning Vec<Opening>
2. Include openings from categories:
   - Open Games (1.e4 e5): Italian, Ruy Lopez, Scotch, King's Gambit, Petrov, Four Knights, Vienna, Bishop's Opening
   - Semi-Open (1.e4 other): Sicilian (+Najdorf, Dragon), French, Caro-Kann, Pirc, Alekhine, Scandinavian
   - Closed (1.d4 d5): Queen's Gambit (+Declined, Accepted), Slav, London, Colle
   - Indian (1.d4 Nf6): King's Indian, Nimzo-Indian, Queen's Indian, Grunfeld, Bogo-Indian, Catalan
   - Flank: English, Reti, Bird, Larsen, King's Indian Attack
   - Gambits: Evans, Smith-Morra, Budapest, Benko, Danish, Blackmar-Diemer
   - Other: Dutch, Benoni, Trompowsky, Torre, Modern, Philidor
3. All openings have ECO codes, valid UCI moves, tags
4. Tests for uniqueness and completeness

---

## Task 5: Add opening lookup by move sequence

**Files:**
- Modify: `crates/chess-openings/src/database.rs`

**Requirements:**
1. find_by_moves(moves) - longest matching opening for move sequence
2. find_all_by_moves(moves) - all openings that are prefixes of sequence
3. continuations_from(moves) - openings that could follow from position
4. Tests for all three methods

---

## Task 6: Integrate with bot-arena

**Files:**
- Modify: `crates/bot-arena/Cargo.toml`
- Modify: `crates/bot-arena/src/main.rs`

**Requirements:**
1. Add chess-openings dependency
2. Add `bot-arena openings` command with --search, --eco, --tag filters
3. Add `--opening` flag to match command
4. Pass opening moves to game runner

---

## Task 7: Add opening detection to game results

**Files:**
- Modify: `crates/bot-arena/src/game_runner.rs`
- Modify: `crates/bot-arena/src/json_output.rs`
- Modify: `crates/bot-arena/src/pgn.rs`

**Requirements:**
1. Detect opening from game moves using find_by_moves()
2. Include opening name/ECO in JSON output
3. Include opening in PGN header
4. Update tests

---

## Summary

After completing all 7 tasks:

**chess-openings crate:**
- Opening struct with ECO codes, moves, tags
- OpeningDatabase with search, filter, random selection
- ~50 built-in popular openings
- Move sequence matching for detection

**bot-arena integration:**
- `bot-arena openings` command
- `--opening` flag for matches
- Automatic opening detection in results

**Usage:**
```bash
bot-arena openings --search "sicilian"
bot-arena openings --tag "gambit"
bot-arena match minimax random --games 10 --opening "italian-game"
```
