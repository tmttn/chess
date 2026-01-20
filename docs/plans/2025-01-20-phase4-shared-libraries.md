# Phase 4: Shared Libraries Design

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Extract reusable, publishable npm packages from the existing chess devtools UI.

**Packages:**
- `@tmttn-chess/board` - Svelte board rendering components
- `@tmttn-chess/game-store` - Game state management
- `@tmttn-chess/bot-client` - WebSocket bot-bridge client

**Design Principles:**
- Loosely coupled via callbacks/interfaces
- WASM engine as peer dependency
- Sounds optional and configurable
- Svelte 5 compatible stores (also work in vanilla JS)

---

## Package Structure

```
packages/
├── board/                 # @tmttn-chess/board
│   ├── src/
│   │   ├── Board.svelte
│   │   ├── Square.svelte
│   │   ├── Piece.svelte
│   │   ├── PromotionDialog.svelte
│   │   ├── types.ts
│   │   └── index.ts
│   ├── package.json       # svelte-package build
│   └── tsconfig.json
│
├── game-store/            # @tmttn-chess/game-store
│   ├── src/
│   │   ├── store.ts       # Main store logic
│   │   ├── types.ts       # Interfaces
│   │   ├── sounds.ts      # Optional sound helpers
│   │   └── index.ts
│   ├── package.json       # tsup build
│   └── tsconfig.json
│
└── bot-client/            # @tmttn-chess/bot-client
    ├── src/
    │   ├── client.ts      # WebSocket client
    │   ├── types.ts       # Message types, SearchInfo
    │   └── index.ts
    ├── package.json       # tsup build
    └── tsconfig.json
```

---

## Package APIs

### @tmttn-chess/game-store

**Initialization:**

```typescript
import { createGameStore, lichessSounds } from '@tmttn-chess/game-store';
import initWasm, { Game } from 'chess-wasm';

await initWasm();

const gameStore = createGameStore({
  createGame: () => new Game(),
  loadFen: (fen) => Game.from_fen(fen),
  sounds: lichessSounds(),  // optional
});
```

**Store Interface:**

```typescript
interface GameStore {
  // Readable stores
  board: Readable<Map<string, PieceInfo>>;
  legalMoves: Readable<Move[]>;
  sideToMove: Readable<'white' | 'black'>;
  isCheck: Readable<boolean>;
  isGameOver: Readable<boolean>;
  moveHistory: Readable<MoveHistoryEntry[]>;
  viewIndex: Readable<number>;

  // Actions
  makeMove(uci: string): boolean;
  newGame(): void;
  loadFen(fen: string): boolean;

  // History navigation
  viewMove(index: number): void;
  viewNext(): void;
  viewPrev(): void;
  viewLive(): void;

  // Bot integration
  attachSearchInfo(info: SearchInfo): void;
}
```

---

### @tmttn-chess/bot-client

**Initialization:**

```typescript
import { createBotClient } from '@tmttn-chess/bot-client';

const client = createBotClient({
  url: 'ws://127.0.0.1:9999',
  onConnect: () => console.log('Connected'),
  onDisconnect: () => console.log('Disconnected'),
  onError: (msg) => console.error(msg),
  onBots: (bots) => console.log('Available:', bots),
  onSearchInfo: (info) => updateUI(info),
  onBestMove: (move, info) => gameStore.makeMove(move),
});
```

**Client Interface:**

```typescript
interface BotClient {
  // State
  connected: Readable<boolean>;
  connecting: Readable<boolean>;
  availableBots: Readable<string[]>;
  searchInfo: Readable<SearchInfo | null>;

  // Connection
  connect(): Promise<void>;
  disconnect(): void;

  // Sessions
  startSession(botName: string): Promise<BotSession>;
}

interface BotSession {
  sendPosition(moves: string[]): void;
  go(options: { movetime?: number; depth?: number }): void;
  stop(): void;
  close(): void;
}

interface SearchInfo {
  depth?: number;
  score?: { cp?: number; mate?: number };
  nodes?: number;
  nps?: number;
  time?: number;
  pv?: string[];
}
```

---

### @tmttn-chess/board

**Usage:**

```svelte
<script>
  import { Board } from '@tmttn-chess/board';
</script>

<Board
  board={boardMap}
  legalMoves={moves}
  lastMove={{ from: 'e2', to: 'e4' }}
  check="e1"
  flipped={false}
  onMove={(from, to, promotion) => handleMove(from, to, promotion)}
/>
```

**Exports:**

```typescript
export { default as Board } from './Board.svelte';
export { default as Square } from './Square.svelte';
export { default as Piece } from './Piece.svelte';
export { default as PromotionDialog } from './PromotionDialog.svelte';
export type { PieceInfo, Move, BoardProps } from './types';
```

**Theming:** CSS custom properties for colors (`--board-light`, `--board-dark`, etc.)

---

## Build Configuration

| Package | Tool | Output |
|---------|------|--------|
| board | svelte-package | .svelte + .d.ts |
| game-store | tsup | ESM + CJS + .d.ts |
| bot-client | tsup | ESM + CJS + .d.ts |

**Example package.json (game-store):**

```json
{
  "name": "@tmttn-chess/game-store",
  "version": "0.1.0",
  "type": "module",
  "exports": {
    ".": {
      "import": "./dist/index.js",
      "types": "./dist/index.d.ts"
    }
  },
  "files": ["dist"],
  "peerDependencies": {
    "svelte": "^5.0.0"
  },
  "scripts": {
    "build": "tsup src/index.ts --format esm --dts",
    "test": "vitest"
  }
}
```

---

## Implementation Tasks

### Task 1: Setup packages workspace
- Add `"workspaces": ["packages/*", "apps/*"]` to root package.json
- Create `packages/` directory structure
- Configure shared tsconfig base

### Task 2: Extract game-store
- Create package scaffold with tsup
- Port store.ts with dependency injection for WASM
- Port sounds.ts as optional helper (lichessSounds)
- Define TypeScript interfaces
- Add tests with Vitest
- Document README with usage examples

### Task 3: Extract bot-client
- Create package scaffold with tsup
- Refactor to event-driven API (no game-store import)
- Define BotClient and BotSession interfaces
- Add tests
- Document README

### Task 4: Extract board
- Create package scaffold with svelte-package
- Port Board, Square, Piece, PromotionDialog
- Convert from store imports to props
- Add CSS custom properties for theming
- Add tests with @testing-library/svelte
- Document README

### Task 5: Update devtools app
- Install workspace packages as dependencies
- Refactor to use new package APIs
- Verify all functionality works

### Task 6: Prepare for publishing
- Add changesets for versioning
- Write comprehensive READMEs
- Add LICENSE files
- Test publish with --dry-run

---

## Integration Example

```svelte
<script>
  import { Board } from '@tmttn-chess/board';
  import { createGameStore, lichessSounds } from '@tmttn-chess/game-store';
  import { createBotClient } from '@tmttn-chess/bot-client';
  import initWasm, { Game } from 'chess-wasm';

  let ready = $state(false);
  let game;
  let bot;

  $effect(() => {
    initWasm().then(() => {
      game = createGameStore({
        createGame: () => new Game(),
        loadFen: (fen) => Game.from_fen(fen),
        sounds: lichessSounds(),
      });

      bot = createBotClient({
        url: 'ws://127.0.0.1:9999',
        onBestMove: (move, info) => {
          game.makeMove(move);
          game.attachSearchInfo(info);
        },
      });

      ready = true;
    });
  });
</script>

{#if ready}
  <Board
    board={$game.board}
    legalMoves={$game.legalMoves}
    onMove={(from, to, promo) => game.makeMove(`${from}${to}${promo ?? ''}`)}
  />
{/if}
```

---

*Last updated: 2025-01-20*
