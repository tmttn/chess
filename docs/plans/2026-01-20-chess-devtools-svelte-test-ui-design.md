# Chess Devtools - Svelte Test UI

**Date:** 2026-01-20
**Purpose:** Manual testing UI for the chess engine
**Framework:** Svelte 5 + TypeScript + Vite

## Overview

A developer-focused web UI for manually testing the chess engine. Prioritizes debugging ergonomics over polish.

## Project Structure

```
apps/
└── web/
    └── chess-devtools/
        └── svelte-test-ui/
            ├── src/
            │   ├── lib/
            │   │   ├── components/
            │   │   │   ├── Board.svelte
            │   │   │   ├── Square.svelte
            │   │   │   ├── Piece.svelte
            │   │   │   ├── MoveHistory.svelte
            │   │   │   ├── GameControls.svelte
            │   │   │   └── DebugPanel.svelte
            │   │   ├── stores/
            │   │   │   └── game.ts
            │   │   ├── wasm/
            │   │   │   └── index.ts
            │   │   └── pieces/
            │   │       └── index.ts
            │   ├── App.svelte
            │   └── main.ts
            ├── public/
            ├── package.json
            ├── svelte.config.js
            ├── vite.config.ts
            └── tsconfig.json
```

## Component Architecture

### Board.svelte
- 8x8 grid of Square components
- Manages selected square state and legal move highlights
- Handles both drag-and-drop and click-click interactions
- Flippable (white/black perspective)
- Coordinates displayed on edges (a-h, 1-8)

### Square.svelte
- Props: file, rank, piece, isLight, isSelected, isLegalTarget, isLastMove
- Emits click and drag events to parent
- Visual states: normal, selected, legal move (dot), last move highlight

### Piece.svelte
- Props: pieceType, color
- Clean, geometric inline SVG pieces
- Draggable with ghost image during drag

### Interaction Flow
1. Click/drag on piece → Board marks square as selected, calculates legal destinations
2. Legal squares highlighted with subtle dots
3. Click/drop on legal square → Make move via WASM, update store
4. Click elsewhere or same square → Deselect

## State Management

### GameState Interface
```typescript
interface GameState {
  fen: string;
  legalMoves: Move[];
  moveHistory: string[];
  isCheck: boolean;
  isGameOver: boolean;
  result: string | null;
  canClaimDraw: boolean;
  halfmoveClock: number;
  fullmoveNumber: number;
  positionHash: string;
  sideToMove: "white" | "black";
}
```

### WASM Wrapper
- `initWasm()` - loads and initializes the WASM module
- `createGame()` - returns new Game instance
- `makeMove(game, uci)` - applies move, returns updated state
- `loadFen(fen)` - creates game from FEN
- `getLegalMoves(game)` - returns array of legal moves

## Debug Panel

### FEN Section
- Text input for FEN string
- Load and Copy buttons
- Validation error display

### Game State Section
- Side to move indicator
- Check status
- Game result
- Draw claim availability
- Halfmove clock
- Position hash (truncated)

### Legal Moves Section
- Scrollable list: `e4 (e2e4)` format
- Click to play move
- Move count displayed

## Move History & Controls

### MoveHistory.svelte
- Two-column: `1. e4 e5`
- Auto-scroll to latest
- Result display at end

### GameControls.svelte
- New Game, Flip Board, Undo, Claim Draw
- Claim Draw enabled only when applicable

## Layout

```
┌─────────────────────────────────────────────────┐
│  [New] [Flip] [Undo] [Claim Draw]               │
├───────────────────────┬─────────────────────────┤
│                       │  Move History           │
│                       │  1. e4    e5            │
│      Chess Board      │  2. Nf3   Nc6           │
│        (8x8)          │  ...                    │
│                       ├─────────────────────────┤
│                       │  Debug Panel            │
│                       │  FEN: [input] [Load]    │
│                       │  Check: No              │
│                       │  Legal moves: 20        │
└───────────────────────┴─────────────────────────┘
```

## Tech Stack

- Svelte 5 + TypeScript
- Vite (bundling, WASM support)
- wasm-pack (chess-wasm build)
- Inline SVG pieces (no external deps)
- CSS custom properties for theming

## Implementation Order

1. Project scaffolding (Vite + Svelte + TypeScript)
2. WASM integration and store setup
3. Board and pieces (visual only)
4. Click-to-move interaction
5. Drag-and-drop interaction
6. Move history panel
7. Game controls
8. Debug panel
