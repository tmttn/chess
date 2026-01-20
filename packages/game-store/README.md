# @chess/game-store

Svelte store for managing chess game state. Framework-agnostic game logic via adapters.

## Installation

```bash
npm install @chess/game-store
```

## Usage

```typescript
import { createGameStore, lichessSounds, preloadSounds } from '@chess/game-store';

// Create store with your WASM game adapter
const gameStore = createGameStore({
  createGame: () => new ChessGame(),
  loadFen: (fen) => ChessGame.fromFen(fen),
  getLegalMoves: (game) => game.legalMoves(),
  makeMove: (game, uci) => game.makeMove(uci),
  getBoardState: (game) => game.getBoard(),
  moveToSan: (game, uci) => game.toSan(uci),
  sounds: lichessSounds()
});

// Initialize
preloadSounds();
gameStore.init();

// Make moves
gameStore.makeMove('e2e4');

// Navigate history
gameStore.viewPrev();
gameStore.viewNext();
gameStore.goToMove(5);
gameStore.goToLive();
```

## API

### `createGameStore(config: GameStoreConfig): GameStore`

Creates a new game store instance.

#### GameStoreConfig

```typescript
interface GameStoreConfig {
  createGame: () => GameAdapter;
  loadFen: (fen: string) => GameAdapter | null;
  getLegalMoves: (game: GameAdapter) => Move[];
  makeMove: (game: GameAdapter, uci: string) => boolean;
  getBoardState: (game: GameAdapter) => Map<string, PieceInfo>;
  moveToSan?: (game: GameAdapter, uci: string) => string | null;
  parseUci?: (uci: string) => { from: string; to: string };
  sounds?: SoundConfig;
}
```

#### GameAdapter Interface

Your game implementation must satisfy this interface:

```typescript
interface GameAdapter {
  toFen(): string;
  isCheck(): boolean;
  isGameOver(): boolean;
  result(): string | null;
  sideToMove(): string;
}
```

### GameStore

#### Derived Stores

| Store | Type | Description |
|-------|------|-------------|
| `board` | `Readable<Map<string, PieceInfo>>` | Current board position |
| `legalMoves` | `Readable<Move[]>` | Legal moves from current position |
| `sideToMove` | `Readable<'white' \| 'black'>` | Current side to move |
| `isCheck` | `Readable<boolean>` | Whether current side is in check |
| `isGameOver` | `Readable<boolean>` | Whether game has ended |
| `moveHistory` | `Readable<MoveHistoryEntry[]>` | Full move history |
| `viewIndex` | `Readable<number>` | Current viewing position (-1 = start) |
| `isViewingHistory` | `Readable<boolean>` | Whether viewing a past position |
| `liveFen` | `Readable<string>` | FEN of current game position |
| `viewFen` | `Readable<string>` | FEN of currently viewed position |
| `viewSearchInfo` | `Readable<MoveSearchInfo \| null>` | Engine info for viewed move |

#### Methods

| Method | Description |
|--------|-------------|
| `init()` | Initialize the game store |
| `makeMove(uci: string): boolean` | Make a move, returns success |
| `newGame()` | Start a new game |
| `loadFen(fen: string): boolean` | Load position from FEN |
| `viewPrev()` | View previous move |
| `viewNext()` | View next move |
| `goToMove(index: number)` | Jump to specific move |
| `goToStart()` | Go to starting position |
| `goToLive()` | Return to current position |
| `attachSearchInfoToLastMove(info)` | Attach engine analysis to last move |

### Sound Functions

```typescript
// Create Lichess-style sounds
const sounds = lichessSounds();

// Preload sounds for instant playback
preloadSounds();

// Mute state (Svelte store)
import { isMuted, toggleMute } from '@chess/game-store';

$isMuted; // boolean
toggleMute();
```

## TypeScript Types

```typescript
import type {
  PieceInfo,
  Move,
  MoveHistoryEntry,
  MoveSearchInfo,
  GameAdapter,
  GameStoreConfig,
  GameStore,
  SoundConfig
} from '@chess/game-store';
```
