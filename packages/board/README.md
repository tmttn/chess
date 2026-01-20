# @chess/board

Svelte 5 chess board component with drag-and-drop and click-to-move support.

## Installation

```bash
npm install @chess/board
```

## Usage

```svelte
<script lang="ts">
  import { Board } from '@chess/board';
  import type { PieceInfo, Move, LastMove } from '@chess/board';

  let board: Map<string, PieceInfo> = $state(new Map());
  let legalMoves: Move[] = $state([]);
  let lastMove: LastMove | null = $state(null);
  let sideToMove: 'white' | 'black' = $state('white');

  function handleMove(from: string, to: string, promotion?: string) {
    const uci = from + to + (promotion ?? '');
    console.log('Move:', uci);
  }
</script>

<Board
  {board}
  {legalMoves}
  {lastMove}
  {sideToMove}
  onMove={handleMove}
/>
```

## Components

### Board

The main chess board component.

```svelte
<Board
  board={boardMap}
  legalMoves={moves}
  flipped={false}
  lastMove={lastMove}
  check={checkSquare}
  sideToMove="white"
  onMove={handleMove}
/>
```

#### Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `board` | `Map<string, PieceInfo>` | required | Board position |
| `legalMoves` | `Move[]` | required | Legal moves array |
| `flipped` | `boolean` | `false` | Flip board for black |
| `lastMove` | `LastMove \| null` | `null` | Highlight last move |
| `check` | `string \| null` | `null` | Square with king in check |
| `sideToMove` | `'white' \| 'black'` | `'white'` | Side to move (for interaction) |
| `onMove` | `(from, to, promotion?) => void` | - | Move callback |

### Square, Piece, PromotionDialog

These components are exported but typically used internally by the Board component.

## CSS Custom Properties

Style the board using CSS custom properties:

```css
:root {
  /* Square colors */
  --square-light: #f0d9b5;
  --square-dark: #b58863;
  --square-selected: #829769;
  --square-last-move: rgba(155, 199, 0, 0.41);
  --square-check: #e64a4a;
  --square-legal: rgba(0, 0, 0, 0.12);

  /* Board sizing */
  --square-size: 60px;

  /* Theme colors (used by promotion dialog) */
  --bg-secondary: #2a2a2a;
  --bg-tertiary: #3a3a3a;
  --text-secondary: #aaa;
  --text-muted: #888;
  --accent: #7fa650;
  --radius: 8px;
  --radius-sm: 4px;
}
```

### Example: Custom Theme

```css
.my-board {
  --square-light: #eeeed2;
  --square-dark: #769656;
  --square-selected: #baca44;
  --square-size: 80px;
}
```

```svelte
<div class="my-board">
  <Board {board} {legalMoves} />
</div>
```

## TypeScript Types

```typescript
import type {
  PieceInfo,
  Move,
  LastMove,
  BoardProps
} from '@chess/board';
```

### PieceInfo

```typescript
interface PieceInfo {
  type: 'p' | 'n' | 'b' | 'r' | 'q' | 'k';
  color: 'white' | 'black';
}
```

### Move

```typescript
interface Move {
  from: string;
  to: string;
  uci: string;
  promotion?: string;
}
```

### LastMove

```typescript
interface LastMove {
  from: string;
  to: string;
}
```

### BoardProps

```typescript
interface BoardProps {
  board: Map<string, PieceInfo>;
  legalMoves: Move[];
  flipped?: boolean;
  lastMove?: LastMove | null;
  check?: string | null;
  sideToMove?: 'white' | 'black';
  onMove?: (from: string, to: string, promotion?: string) => void;
}
```

## Features

- Click-to-move: Click a piece to select, then click destination
- Drag-and-drop: Drag pieces to make moves
- Legal move hints: Dots on empty squares, rings on capturable pieces
- Promotion dialog: Automatic popup for pawn promotion
- Last move highlight: Yellow highlight on from/to squares
- Check highlight: Red highlight on king in check
- Keyboard support: Enter key to select/move
- Coordinates: File and rank labels
