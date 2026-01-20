# Phase 4: Shared Libraries Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Extract reusable npm packages from the chess devtools UI for publishing.

**Architecture:** Three packages (`@tmttn-chess/game-store`, `@tmttn-chess/bot-client`, `@tmttn-chess/board`) extracted into a `packages/` directory using npm workspaces. The devtools app becomes a consumer of these packages.

**Tech Stack:** TypeScript, Svelte 5, tsup (for TS packages), svelte-package (for Svelte components), Vitest for testing.

---

## Task 1: Setup npm workspaces structure

**Files:**
- Create: `package.json` (root)
- Create: `packages/.gitkeep`
- Modify: `apps/web/chess-devtools/svelte-test-ui/package.json`

**Step 1: Create root package.json for workspaces**

```json
{
  "name": "chess-monorepo",
  "private": true,
  "workspaces": [
    "packages/*",
    "apps/web/chess-devtools/svelte-test-ui"
  ],
  "scripts": {
    "build": "npm run build --workspaces",
    "test": "npm run test --workspaces --if-present"
  },
  "devDependencies": {
    "typescript": "^5.9.0"
  }
}
```

**Step 2: Create packages directory**

```bash
mkdir -p packages
touch packages/.gitkeep
```

**Step 3: Verify workspace setup**

Run: `npm install`
Expected: npm recognizes workspaces structure

**Step 4: Commit**

```bash
git add package.json packages/
git commit -m "chore: setup npm workspaces for shared packages"
```

---

## Task 2: Create game-store package scaffold

**Files:**
- Create: `packages/game-store/package.json`
- Create: `packages/game-store/tsconfig.json`
- Create: `packages/game-store/src/index.ts`
- Create: `packages/game-store/src/types.ts`

**Step 1: Create package.json**

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
  "scripts": {
    "build": "tsup src/index.ts --format esm --dts",
    "test": "vitest run",
    "test:watch": "vitest"
  },
  "peerDependencies": {
    "svelte": "^5.0.0"
  },
  "devDependencies": {
    "svelte": "^5.0.0",
    "tsup": "^8.0.0",
    "vitest": "^2.0.0"
  }
}
```

**Step 2: Create tsconfig.json**

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "strict": true,
    "declaration": true,
    "outDir": "dist",
    "rootDir": "src",
    "esModuleInterop": true,
    "skipLibCheck": true
  },
  "include": ["src"]
}
```

**Step 3: Create types.ts with interfaces**

```typescript
// packages/game-store/src/types.ts

import type { Readable } from 'svelte/store';

/** Information about a piece on the board */
export interface PieceInfo {
  type: 'p' | 'n' | 'b' | 'r' | 'q' | 'k';
  color: 'white' | 'black';
}

/** A legal move in UCI format with additional info */
export interface Move {
  from: string;
  to: string;
  uci: string;
  promotion?: string;
}

/** Search info attached to a move from an engine */
export interface MoveSearchInfo {
  depth: number;
  score: number;
  nodes: number;
  time: number;
  pv: string[];
}

/** A single move in the game history */
export interface MoveHistoryEntry {
  uci: string;
  san: string;
  fen: string;
  searchInfo?: MoveSearchInfo;
}

/** WASM Game abstraction - consumer provides implementation */
export interface GameAdapter {
  toFen(): string;
  isCheck(): boolean;
  isGameOver(): boolean;
  result(): string | null;
  sideToMove(): string;
}

/** Factory functions the consumer provides */
export interface GameStoreConfig {
  createGame: () => GameAdapter;
  loadFen: (fen: string) => GameAdapter | null;
  getLegalMoves: (game: GameAdapter) => Move[];
  makeMove: (game: GameAdapter, uci: string) => boolean;
  getBoardState: (game: GameAdapter) => Map<string, PieceInfo>;
  moveToSan?: (game: GameAdapter, uci: string) => string | null;
  parseUci?: (uci: string) => { from: string; to: string };
  sounds?: SoundConfig;
}

/** Optional sound configuration */
export interface SoundConfig {
  playMove: () => void;
  playCapture: () => void;
  playCheck: () => void;
  playGameStart: () => void;
  playGameEnd: () => void;
}

/** The public game store interface */
export interface GameStore {
  // Readable stores
  board: Readable<Map<string, PieceInfo>>;
  legalMoves: Readable<Move[]>;
  sideToMove: Readable<'white' | 'black'>;
  isCheck: Readable<boolean>;
  isGameOver: Readable<boolean>;
  moveHistory: Readable<MoveHistoryEntry[]>;
  viewIndex: Readable<number>;
  isViewingHistory: Readable<boolean>;
  liveFen: Readable<string>;
  viewFen: Readable<string>;
  viewSearchInfo: Readable<MoveSearchInfo | null>;

  // Actions
  init(): void;
  makeMove(uci: string): boolean;
  newGame(): void;
  loadFen(fen: string): boolean;

  // History navigation
  viewPrev(): void;
  viewNext(): void;
  goToMove(index: number): void;
  goToStart(): void;
  goToLive(): void;

  // Bot integration
  attachSearchInfoToLastMove(searchInfo: MoveSearchInfo): void;
}
```

**Step 4: Create index.ts with placeholder export**

```typescript
// packages/game-store/src/index.ts

export * from './types';

// Store implementation will be added in Task 3
export function createGameStore(config: import('./types').GameStoreConfig): import('./types').GameStore {
  throw new Error('Not implemented yet');
}
```

**Step 5: Install dependencies and build**

Run: `cd packages/game-store && npm install && npm run build`
Expected: Build succeeds with dist/ created

**Step 6: Commit**

```bash
git add packages/game-store/
git commit -m "feat(game-store): add package scaffold with type definitions"
```

---

## Task 3: Implement game-store logic

**Files:**
- Create: `packages/game-store/src/store.ts`
- Modify: `packages/game-store/src/index.ts`

**Step 1: Create store.ts with full implementation**

```typescript
// packages/game-store/src/store.ts

import { writable, derived, type Readable } from 'svelte/store';
import type {
  GameAdapter,
  GameStore,
  GameStoreConfig,
  Move,
  MoveHistoryEntry,
  MoveSearchInfo,
  PieceInfo,
} from './types';

const STARTING_FEN = 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1';

interface GameState {
  game: GameAdapter | null;
  fen: string;
  legalMoves: Move[];
  board: Map<string, PieceInfo>;
  liveBoard: Map<string, PieceInfo>;
  moveHistory: MoveHistoryEntry[];
  viewIndex: number;
  isCheck: boolean;
  isGameOver: boolean;
  result: string | null;
  sideToMove: 'white' | 'black';
}

const initialState: GameState = {
  game: null,
  fen: STARTING_FEN,
  legalMoves: [],
  board: new Map(),
  liveBoard: new Map(),
  moveHistory: [],
  viewIndex: -1,
  isCheck: false,
  isGameOver: false,
  result: null,
  sideToMove: 'white',
};

export function createGameStore(config: GameStoreConfig): GameStore {
  const { subscribe, set, update } = writable<GameState>(initialState);

  function refreshState(game: GameAdapter): Partial<GameState> {
    const board = config.getBoardState(game);
    return {
      fen: game.toFen(),
      legalMoves: config.getLegalMoves(game),
      board: board,
      liveBoard: board,
      isCheck: game.isCheck(),
      isGameOver: game.isGameOver(),
      result: game.result() ?? null,
      sideToMove: game.sideToMove() as 'white' | 'black',
    };
  }

  function getBoardForFen(fen: string): Map<string, PieceInfo> {
    const tempGame = config.loadFen(fen);
    if (!tempGame) return new Map();
    return config.getBoardState(tempGame);
  }

  // Default parseUci if not provided
  const parseUci = config.parseUci ?? ((uci: string) => ({
    from: uci.slice(0, 2),
    to: uci.slice(2, 4),
  }));

  const store = {
    subscribe,

    init() {
      config.sounds?.playGameStart();
      const game = config.createGame();
      set({
        ...initialState,
        game,
        ...refreshState(game),
      });
    },

    makeMove(uci: string): boolean {
      let success = false;
      update((state) => {
        if (!state.game) return state;

        const move = parseUci(uci);
        const isCapture = state.liveBoard.has(move.to);
        const san = config.moveToSan?.(state.game, uci) ?? uci;

        if (config.makeMove(state.game, uci)) {
          success = true;
          const newState = refreshState(state.game);

          // Play appropriate sound
          if (newState.isGameOver) {
            config.sounds?.playGameEnd();
          } else if (newState.isCheck) {
            config.sounds?.playCheck();
          } else if (isCapture) {
            config.sounds?.playCapture();
          } else {
            config.sounds?.playMove();
          }

          const newHistory = [...state.moveHistory, { uci, san, fen: newState.fen! }];
          return {
            ...state,
            ...newState,
            moveHistory: newHistory,
            viewIndex: newHistory.length - 1,
          };
        }
        return state;
      });
      return success;
    },

    viewPrev() {
      update((state) => {
        if (state.viewIndex < 0) return state;
        const newIndex = state.viewIndex - 1;
        const fenToView = newIndex >= 0 ? state.moveHistory[newIndex]!.fen : STARTING_FEN;
        return {
          ...state,
          board: getBoardForFen(fenToView),
          viewIndex: newIndex,
        };
      });
    },

    viewNext() {
      update((state) => {
        if (state.viewIndex >= state.moveHistory.length - 1) return state;
        const newIndex = state.viewIndex + 1;
        const fenToView = state.moveHistory[newIndex]!.fen;
        return {
          ...state,
          board: getBoardForFen(fenToView),
          viewIndex: newIndex,
        };
      });
    },

    goToMove(index: number) {
      update((state) => {
        if (index < -1 || index >= state.moveHistory.length) return state;
        const fenToView = index >= 0 ? state.moveHistory[index]!.fen : STARTING_FEN;
        return {
          ...state,
          board: getBoardForFen(fenToView),
          viewIndex: index,
        };
      });
    },

    goToStart() {
      update((state) => ({
        ...state,
        board: getBoardForFen(STARTING_FEN),
        viewIndex: -1,
      }));
    },

    goToLive() {
      update((state) => ({
        ...state,
        board: state.liveBoard,
        viewIndex: state.moveHistory.length - 1,
      }));
    },

    newGame() {
      const game = config.createGame();
      set({
        ...initialState,
        game,
        ...refreshState(game),
      });
      config.sounds?.playGameStart();
    },

    loadFen(fen: string): boolean {
      const newGame = config.loadFen(fen);
      if (!newGame) return false;
      set({
        ...initialState,
        game: newGame,
        ...refreshState(newGame),
      });
      return true;
    },

    attachSearchInfoToLastMove(searchInfo: MoveSearchInfo) {
      update((state) => {
        if (state.moveHistory.length === 0) return state;
        const newHistory = [...state.moveHistory];
        const lastIndex = newHistory.length - 1;
        newHistory[lastIndex] = { ...newHistory[lastIndex], searchInfo };
        return { ...state, moveHistory: newHistory };
      });
    },
  };

  // Derived stores
  const board: Readable<Map<string, PieceInfo>> = derived({ subscribe }, ($s) => $s.board);
  const legalMoves: Readable<Move[]> = derived({ subscribe }, ($s) => $s.legalMoves);
  const sideToMove: Readable<'white' | 'black'> = derived({ subscribe }, ($s) => $s.sideToMove);
  const isCheck: Readable<boolean> = derived({ subscribe }, ($s) => $s.isCheck);
  const isGameOver: Readable<boolean> = derived({ subscribe }, ($s) => $s.isGameOver);
  const moveHistory: Readable<MoveHistoryEntry[]> = derived({ subscribe }, ($s) => $s.moveHistory);
  const viewIndex: Readable<number> = derived({ subscribe }, ($s) => $s.viewIndex);
  const isViewingHistory: Readable<boolean> = derived(
    { subscribe },
    ($s) => $s.viewIndex < $s.moveHistory.length - 1
  );
  const liveFen: Readable<string> = derived({ subscribe }, ($s) => $s.fen);
  const viewFen: Readable<string> = derived({ subscribe }, ($s) =>
    $s.viewIndex >= 0 ? $s.moveHistory[$s.viewIndex]?.fen ?? STARTING_FEN : STARTING_FEN
  );
  const viewSearchInfo: Readable<MoveSearchInfo | null> = derived({ subscribe }, ($s) =>
    $s.viewIndex >= 0 ? $s.moveHistory[$s.viewIndex]?.searchInfo ?? null : null
  );

  return {
    ...store,
    board,
    legalMoves,
    sideToMove,
    isCheck,
    isGameOver,
    moveHistory,
    viewIndex,
    isViewingHistory,
    liveFen,
    viewFen,
    viewSearchInfo,
  };
}
```

**Step 2: Update index.ts to export from store**

```typescript
// packages/game-store/src/index.ts

export * from './types';
export { createGameStore } from './store';
```

**Step 3: Build and verify**

Run: `cd packages/game-store && npm run build`
Expected: Build succeeds

**Step 4: Commit**

```bash
git add packages/game-store/
git commit -m "feat(game-store): implement createGameStore with dependency injection"
```

---

## Task 4: Add game-store tests

**Files:**
- Create: `packages/game-store/src/store.test.ts`
- Create: `packages/game-store/vitest.config.ts`

**Step 1: Create vitest config**

```typescript
// packages/game-store/vitest.config.ts

import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    environment: 'node',
  },
});
```

**Step 2: Create test file**

```typescript
// packages/game-store/src/store.test.ts

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import { createGameStore } from './store';
import type { GameAdapter, GameStoreConfig, Move, PieceInfo } from './types';

// Mock game adapter
function createMockGame(fen = 'startpos'): GameAdapter {
  let currentFen = fen;
  return {
    toFen: () => currentFen,
    isCheck: () => false,
    isGameOver: () => false,
    result: () => null,
    sideToMove: () => 'white',
  };
}

// Mock config
function createMockConfig(): GameStoreConfig {
  return {
    createGame: () => createMockGame(),
    loadFen: (fen) => createMockGame(fen),
    getLegalMoves: () => [
      { from: 'e2', to: 'e4', uci: 'e2e4' },
      { from: 'e2', to: 'e3', uci: 'e2e3' },
    ],
    makeMove: () => true,
    getBoardState: () => new Map([['e2', { type: 'p', color: 'white' }]]),
    moveToSan: () => 'e4',
    sounds: {
      playMove: vi.fn(),
      playCapture: vi.fn(),
      playCheck: vi.fn(),
      playGameStart: vi.fn(),
      playGameEnd: vi.fn(),
    },
  };
}

describe('createGameStore', () => {
  let config: GameStoreConfig;

  beforeEach(() => {
    config = createMockConfig();
  });

  it('should create a store with initial state', () => {
    const store = createGameStore(config);
    expect(get(store.board)).toBeInstanceOf(Map);
    expect(get(store.legalMoves)).toEqual([]);
    expect(get(store.viewIndex)).toBe(-1);
  });

  it('should initialize game state when init() is called', () => {
    const store = createGameStore(config);
    store.init();

    expect(get(store.board).size).toBeGreaterThan(0);
    expect(get(store.legalMoves).length).toBeGreaterThan(0);
    expect(config.sounds?.playGameStart).toHaveBeenCalled();
  });

  it('should make a move and update history', () => {
    const store = createGameStore(config);
    store.init();

    const result = store.makeMove('e2e4');

    expect(result).toBe(true);
    expect(get(store.moveHistory).length).toBe(1);
    expect(get(store.moveHistory)[0].uci).toBe('e2e4');
    expect(get(store.viewIndex)).toBe(0);
    expect(config.sounds?.playMove).toHaveBeenCalled();
  });

  it('should navigate through move history', () => {
    const store = createGameStore(config);
    store.init();
    store.makeMove('e2e4');
    store.makeMove('e7e5');

    expect(get(store.viewIndex)).toBe(1);

    store.viewPrev();
    expect(get(store.viewIndex)).toBe(0);

    store.viewPrev();
    expect(get(store.viewIndex)).toBe(-1);

    store.viewNext();
    expect(get(store.viewIndex)).toBe(0);

    store.goToLive();
    expect(get(store.viewIndex)).toBe(1);

    store.goToStart();
    expect(get(store.viewIndex)).toBe(-1);
  });

  it('should attach search info to last move', () => {
    const store = createGameStore(config);
    store.init();
    store.makeMove('e2e4');

    const searchInfo = { depth: 20, score: 35, nodes: 1000, time: 500, pv: ['e2e4'] };
    store.attachSearchInfoToLastMove(searchInfo);

    expect(get(store.moveHistory)[0].searchInfo).toEqual(searchInfo);
  });

  it('should reset state on newGame()', () => {
    const store = createGameStore(config);
    store.init();
    store.makeMove('e2e4');
    expect(get(store.moveHistory).length).toBe(1);

    store.newGame();

    expect(get(store.moveHistory).length).toBe(0);
    expect(get(store.viewIndex)).toBe(-1);
  });

  it('should report isViewingHistory correctly', () => {
    const store = createGameStore(config);
    store.init();
    store.makeMove('e2e4');

    expect(get(store.isViewingHistory)).toBe(false);

    store.viewPrev();
    expect(get(store.isViewingHistory)).toBe(true);

    store.goToLive();
    expect(get(store.isViewingHistory)).toBe(false);
  });
});
```

**Step 3: Run tests**

Run: `cd packages/game-store && npm test`
Expected: All tests pass

**Step 4: Commit**

```bash
git add packages/game-store/
git commit -m "test(game-store): add unit tests for createGameStore"
```

---

## Task 5: Create bot-client package scaffold

**Files:**
- Create: `packages/bot-client/package.json`
- Create: `packages/bot-client/tsconfig.json`
- Create: `packages/bot-client/src/types.ts`
- Create: `packages/bot-client/src/index.ts`

**Step 1: Create package.json**

```json
{
  "name": "@tmttn-chess/bot-client",
  "version": "0.1.0",
  "type": "module",
  "exports": {
    ".": {
      "import": "./dist/index.js",
      "types": "./dist/index.d.ts"
    }
  },
  "files": ["dist"],
  "scripts": {
    "build": "tsup src/index.ts --format esm --dts",
    "test": "vitest run",
    "test:watch": "vitest"
  },
  "peerDependencies": {
    "svelte": "^5.0.0"
  },
  "devDependencies": {
    "svelte": "^5.0.0",
    "tsup": "^8.0.0",
    "vitest": "^2.0.0"
  }
}
```

**Step 2: Create tsconfig.json**

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "strict": true,
    "declaration": true,
    "outDir": "dist",
    "rootDir": "src",
    "esModuleInterop": true,
    "skipLibCheck": true
  },
  "include": ["src"]
}
```

**Step 3: Create types.ts**

```typescript
// packages/bot-client/src/types.ts

import type { Readable } from 'svelte/store';

/** Search information from UCI engine */
export interface SearchInfo {
  depth: number;
  score: number;
  nodes: number;
  time: number;
  pv: string[];
}

/** Event callbacks for the bot client */
export interface BotClientCallbacks {
  onConnect?: () => void;
  onDisconnect?: () => void;
  onError?: (message: string) => void;
  onBots?: (bots: string[]) => void;
  onSearchInfo?: (info: SearchInfo) => void;
  onBestMove?: (move: string, searchInfo: SearchInfo | null) => void;
}

/** Configuration for creating a bot client */
export interface BotClientConfig extends BotClientCallbacks {
  url?: string;
}

/** A session with a specific bot */
export interface BotSession {
  name: string;
  sendPosition(moves: string[]): void;
  go(options: GoOptions): void;
  stop(): void;
  close(): void;
}

/** Options for the "go" command */
export interface GoOptions {
  movetime?: number;
  depth?: number;
  infinite?: boolean;
}

/** The public bot client interface */
export interface BotClient {
  // State stores
  connected: Readable<boolean>;
  connecting: Readable<boolean>;
  availableBots: Readable<string[]>;
  searchInfo: Readable<SearchInfo | null>;
  error: Readable<string | null>;

  // Connection methods
  connect(): Promise<void>;
  disconnect(): void;

  // Session management
  startSession(botName: string): Promise<BotSession>;

  // Direct commands (for debugging)
  sendRawCommand(cmd: string, bot?: string): void;
}
```

**Step 4: Create index.ts placeholder**

```typescript
// packages/bot-client/src/index.ts

export * from './types';

export function createBotClient(config: import('./types').BotClientConfig): import('./types').BotClient {
  throw new Error('Not implemented yet');
}
```

**Step 5: Install and build**

Run: `cd packages/bot-client && npm install && npm run build`
Expected: Build succeeds

**Step 6: Commit**

```bash
git add packages/bot-client/
git commit -m "feat(bot-client): add package scaffold with type definitions"
```

---

## Task 6: Implement bot-client logic

**Files:**
- Create: `packages/bot-client/src/client.ts`
- Modify: `packages/bot-client/src/index.ts`

**Step 1: Create client.ts**

```typescript
// packages/bot-client/src/client.ts

import { writable, derived, get, type Readable } from 'svelte/store';
import type {
  BotClient,
  BotClientConfig,
  BotSession,
  GoOptions,
  SearchInfo,
} from './types';

interface ClientState {
  connected: boolean;
  connecting: boolean;
  availableBots: string[];
  activeSessions: Map<string, { name: string; ready: boolean }>;
  searchInfo: SearchInfo | null;
  error: string | null;
}

const initialState: ClientState = {
  connected: false,
  connecting: false,
  availableBots: [],
  activeSessions: new Map(),
  searchInfo: null,
  error: null,
};

function parseInfoLine(line: string): SearchInfo | null {
  if (!line.startsWith('info ')) return null;

  const parts = line.split(' ');
  const info: SearchInfo = { depth: 0, score: 0, nodes: 0, time: 0, pv: [] };

  for (let i = 1; i < parts.length; i++) {
    switch (parts[i]) {
      case 'depth':
        info.depth = parseInt(parts[++i] || '0', 10);
        break;
      case 'score':
        if (parts[i + 1] === 'cp') {
          i++;
          info.score = parseInt(parts[++i] || '0', 10);
        } else if (parts[i + 1] === 'mate') {
          i++;
          const mateIn = parseInt(parts[++i] || '0', 10);
          info.score = mateIn > 0 ? 100000 - mateIn : -100000 - mateIn;
        }
        break;
      case 'nodes':
        info.nodes = parseInt(parts[++i] || '0', 10);
        break;
      case 'time':
        info.time = parseInt(parts[++i] || '0', 10);
        break;
      case 'pv':
        info.pv = parts.slice(i + 1);
        i = parts.length;
        break;
    }
  }

  return info.depth > 0 ? info : null;
}

export function createBotClient(config: BotClientConfig = {}): BotClient {
  const url = config.url ?? 'ws://127.0.0.1:9999';
  const { subscribe, set, update } = writable<ClientState>(initialState);

  let ws: WebSocket | null = null;
  let currentBotTurn: string | null = null;

  function sendToBot(msg: object) {
    if (ws && ws.readyState === WebSocket.OPEN) {
      ws.send(JSON.stringify(msg));
    }
  }

  function handleMessage(data: any) {
    update((state) => {
      switch (data.type) {
        case 'bots':
          config.onBots?.(data.bots);
          config.onConnect?.();
          return {
            ...state,
            availableBots: data.bots,
            connecting: false,
            connected: true,
          };

        case 'connected': {
          const sessions = new Map(state.activeSessions);
          sessions.set(data.bot, { name: data.bot, ready: false });
          return { ...state, activeSessions: sessions };
        }

        case 'disconnected':
          return state;

        case 'error':
          config.onError?.(data.message);
          return {
            ...state,
            error: data.message,
            connecting: false,
          };

        case 'uci': {
          const line = data.line;

          if (line === 'readyok' && currentBotTurn) {
            const sessions = new Map(state.activeSessions);
            const session = sessions.get(currentBotTurn);
            if (session) {
              sessions.set(currentBotTurn, { ...session, ready: true });
            }
            return { ...state, activeSessions: sessions };
          }

          const parsedInfo = parseInfoLine(line);
          if (parsedInfo) {
            config.onSearchInfo?.(parsedInfo);
            return { ...state, searchInfo: parsedInfo };
          }

          if (line.startsWith('bestmove ')) {
            const parts = line.split(' ');
            const move = parts[1];
            if (move && move !== '(none)' && move !== '0000') {
              const lastSearchInfo = state.searchInfo;
              config.onBestMove?.(move, lastSearchInfo);
              currentBotTurn = null;
            }
            return { ...state, searchInfo: null };
          }

          return state;
        }

        default:
          return state;
      }
    });
  }

  async function connect(): Promise<void> {
    return new Promise((resolve, reject) => {
      if (ws) {
        ws.close();
      }

      update((s) => ({ ...s, connecting: true, error: null }));

      ws = new WebSocket(url);

      ws.onopen = () => {
        sendToBot({ type: 'list' });
        resolve();
      };

      ws.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data);
          handleMessage(data);
        } catch (e) {
          console.error('Failed to parse message:', event.data);
        }
      };

      ws.onerror = () => {
        const error = 'WebSocket connection error';
        update((s) => ({ ...s, error, connecting: false }));
        config.onError?.(error);
        reject(new Error(error));
      };

      ws.onclose = () => {
        update((s) => ({
          ...s,
          connected: false,
          connecting: false,
          activeSessions: new Map(),
        }));
        config.onDisconnect?.();
        ws = null;
      };
    });
  }

  function disconnect() {
    if (ws) {
      ws.close();
      ws = null;
    }
    set(initialState);
  }

  async function startSession(botName: string): Promise<BotSession> {
    const state = get({ subscribe });

    if (!state.activeSessions.has(botName)) {
      sendToBot({ type: 'connect', bot: botName });
      await new Promise((resolve) => setTimeout(resolve, 50));
      sendToBot({ type: 'uci', cmd: 'uci', bot: botName });
      sendToBot({ type: 'uci', cmd: 'isready', bot: botName });
      await new Promise((resolve) => setTimeout(resolve, 100));
    }

    return {
      name: botName,
      sendPosition(moves: string[]) {
        const movesStr = moves.length > 0 ? ' moves ' + moves.join(' ') : '';
        sendToBot({ type: 'uci', cmd: `position startpos${movesStr}`, bot: botName });
      },
      go(options: GoOptions) {
        currentBotTurn = botName;
        let cmd = 'go';
        if (options.movetime) cmd += ` movetime ${options.movetime}`;
        if (options.depth) cmd += ` depth ${options.depth}`;
        if (options.infinite) cmd += ' infinite';
        sendToBot({ type: 'uci', cmd, bot: botName });
      },
      stop() {
        sendToBot({ type: 'uci', cmd: 'stop', bot: botName });
      },
      close() {
        sendToBot({ type: 'disconnect', bot: botName });
        update((s) => {
          const sessions = new Map(s.activeSessions);
          sessions.delete(botName);
          return { ...s, activeSessions: sessions };
        });
      },
    };
  }

  function sendRawCommand(cmd: string, bot?: string) {
    sendToBot({ type: 'uci', cmd, bot });
  }

  // Derived stores
  const connected: Readable<boolean> = derived({ subscribe }, ($s) => $s.connected);
  const connecting: Readable<boolean> = derived({ subscribe }, ($s) => $s.connecting);
  const availableBots: Readable<string[]> = derived({ subscribe }, ($s) => $s.availableBots);
  const searchInfo: Readable<SearchInfo | null> = derived({ subscribe }, ($s) => $s.searchInfo);
  const error: Readable<string | null> = derived({ subscribe }, ($s) => $s.error);

  return {
    connected,
    connecting,
    availableBots,
    searchInfo,
    error,
    connect,
    disconnect,
    startSession,
    sendRawCommand,
  };
}
```

**Step 2: Update index.ts**

```typescript
// packages/bot-client/src/index.ts

export * from './types';
export { createBotClient } from './client';
```

**Step 3: Build and verify**

Run: `cd packages/bot-client && npm run build`
Expected: Build succeeds

**Step 4: Commit**

```bash
git add packages/bot-client/
git commit -m "feat(bot-client): implement createBotClient with event-driven API"
```

---

## Task 7: Add bot-client tests

**Files:**
- Create: `packages/bot-client/vitest.config.ts`
- Create: `packages/bot-client/src/client.test.ts`

**Step 1: Create vitest config**

```typescript
// packages/bot-client/vitest.config.ts

import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    environment: 'jsdom',
  },
});
```

**Step 2: Create test file**

```typescript
// packages/bot-client/src/client.test.ts

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { get } from 'svelte/store';
import { createBotClient } from './client';

// Mock WebSocket
class MockWebSocket {
  static instances: MockWebSocket[] = [];
  onopen: (() => void) | null = null;
  onclose: (() => void) | null = null;
  onmessage: ((e: { data: string }) => void) | null = null;
  onerror: (() => void) | null = null;
  readyState = WebSocket.OPEN;
  sentMessages: object[] = [];

  constructor(public url: string) {
    MockWebSocket.instances.push(this);
    setTimeout(() => this.onopen?.(), 0);
  }

  send(data: string) {
    this.sentMessages.push(JSON.parse(data));
  }

  close() {
    this.readyState = WebSocket.CLOSED;
    this.onclose?.();
  }

  simulateMessage(data: object) {
    this.onmessage?.({ data: JSON.stringify(data) });
  }
}

describe('createBotClient', () => {
  beforeEach(() => {
    MockWebSocket.instances = [];
    vi.stubGlobal('WebSocket', MockWebSocket);
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('should start in disconnected state', () => {
    const client = createBotClient();
    expect(get(client.connected)).toBe(false);
    expect(get(client.connecting)).toBe(false);
  });

  it('should connect and receive bot list', async () => {
    const onBots = vi.fn();
    const client = createBotClient({ onBots });

    const connectPromise = client.connect();
    await new Promise((r) => setTimeout(r, 10));

    const ws = MockWebSocket.instances[0];
    ws.simulateMessage({ type: 'bots', bots: ['stockfish', 'random'] });

    await connectPromise;

    expect(get(client.connected)).toBe(true);
    expect(get(client.availableBots)).toEqual(['stockfish', 'random']);
    expect(onBots).toHaveBeenCalledWith(['stockfish', 'random']);
  });

  it('should handle search info callback', async () => {
    const onSearchInfo = vi.fn();
    const client = createBotClient({ onSearchInfo });

    await client.connect();
    await new Promise((r) => setTimeout(r, 10));

    const ws = MockWebSocket.instances[0];
    ws.simulateMessage({
      type: 'uci',
      line: 'info depth 20 score cp 35 nodes 1000 time 500 pv e2e4 e7e5',
    });

    expect(onSearchInfo).toHaveBeenCalledWith({
      depth: 20,
      score: 35,
      nodes: 1000,
      time: 500,
      pv: ['e2e4', 'e7e5'],
    });
  });

  it('should handle bestmove callback', async () => {
    const onBestMove = vi.fn();
    const client = createBotClient({ onBestMove });

    await client.connect();
    await new Promise((r) => setTimeout(r, 10));

    const ws = MockWebSocket.instances[0];

    // Simulate search info first
    ws.simulateMessage({
      type: 'uci',
      line: 'info depth 20 score cp 35 nodes 1000 time 500 pv e2e4',
    });

    // Then bestmove
    ws.simulateMessage({ type: 'uci', line: 'bestmove e2e4' });

    expect(onBestMove).toHaveBeenCalledWith('e2e4', {
      depth: 20,
      score: 35,
      nodes: 1000,
      time: 500,
      pv: ['e2e4'],
    });
  });

  it('should disconnect cleanly', async () => {
    const onDisconnect = vi.fn();
    const client = createBotClient({ onDisconnect });

    await client.connect();
    await new Promise((r) => setTimeout(r, 10));

    client.disconnect();

    expect(get(client.connected)).toBe(false);
  });
});
```

**Step 3: Install jsdom and run tests**

Run: `cd packages/bot-client && npm install -D jsdom && npm test`
Expected: All tests pass

**Step 4: Commit**

```bash
git add packages/bot-client/
git commit -m "test(bot-client): add unit tests for createBotClient"
```

---

## Task 8: Create board package scaffold

**Files:**
- Create: `packages/board/package.json`
- Create: `packages/board/svelte.config.js`
- Create: `packages/board/tsconfig.json`
- Create: `packages/board/src/lib/index.ts`
- Create: `packages/board/src/lib/types.ts`

**Step 1: Create package.json**

```json
{
  "name": "@tmttn-chess/board",
  "version": "0.1.0",
  "type": "module",
  "svelte": "./dist/index.js",
  "types": "./dist/index.d.ts",
  "exports": {
    ".": {
      "svelte": "./dist/index.js",
      "types": "./dist/index.d.ts"
    }
  },
  "files": ["dist"],
  "scripts": {
    "build": "svelte-kit sync && svelte-package",
    "test": "vitest run",
    "test:watch": "vitest"
  },
  "peerDependencies": {
    "svelte": "^5.0.0"
  },
  "devDependencies": {
    "@sveltejs/kit": "^2.50.0",
    "@sveltejs/package": "^2.0.0",
    "@sveltejs/vite-plugin-svelte": "^4.0.0",
    "svelte": "^5.0.0",
    "typescript": "^5.9.0",
    "vite": "^6.0.0",
    "vitest": "^2.0.0"
  }
}
```

**Step 2: Create svelte.config.js**

```javascript
// packages/board/svelte.config.js

import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),
  kit: {
    files: {
      lib: 'src/lib'
    }
  }
};

export default config;
```

**Step 3: Create tsconfig.json**

```json
{
  "extends": "./.svelte-kit/tsconfig.json",
  "compilerOptions": {
    "allowJs": true,
    "checkJs": true,
    "esModuleInterop": true,
    "forceConsistentCasingInFileNames": true,
    "resolveJsonModule": true,
    "skipLibCheck": true,
    "sourceMap": true,
    "strict": true,
    "moduleResolution": "bundler"
  }
}
```

**Step 4: Create types.ts**

```typescript
// packages/board/src/lib/types.ts

/** Information about a piece on the board */
export interface PieceInfo {
  type: 'p' | 'n' | 'b' | 'r' | 'q' | 'k';
  color: 'white' | 'black';
}

/** A legal move with destination */
export interface Move {
  from: string;
  to: string;
  uci: string;
  promotion?: string;
}

/** Last move highlight info */
export interface LastMove {
  from: string;
  to: string;
}

/** Board component props */
export interface BoardProps {
  board: Map<string, PieceInfo>;
  legalMoves: Move[];
  flipped?: boolean;
  lastMove?: LastMove | null;
  check?: string | null;
  sideToMove?: 'white' | 'black';
  onMove?: (from: string, to: string, promotion?: string) => void;
}
```

**Step 5: Create index.ts placeholder**

```typescript
// packages/board/src/lib/index.ts

export * from './types';

// Components will be added in Task 9
```

**Step 6: Create minimal SvelteKit app structure for svelte-package**

```bash
mkdir -p packages/board/src/routes
echo '<slot />' > packages/board/src/routes/+layout.svelte
echo 'export const prerender = true;' > packages/board/src/routes/+layout.ts
echo '<h1>Board Package Dev</h1>' > packages/board/src/routes/+page.svelte
```

**Step 7: Install and build**

Run: `cd packages/board && npm install && npm run build`
Expected: Build succeeds (may need to run `npx svelte-kit sync` first)

**Step 8: Commit**

```bash
git add packages/board/
git commit -m "feat(board): add package scaffold with type definitions"
```

---

## Task 9: Port Board components

**Files:**
- Create: `packages/board/src/lib/Piece.svelte`
- Create: `packages/board/src/lib/Square.svelte`
- Create: `packages/board/src/lib/PromotionDialog.svelte`
- Create: `packages/board/src/lib/Board.svelte`
- Modify: `packages/board/src/lib/index.ts`

**Step 1: Create Piece.svelte**

Copy from `apps/web/chess-devtools/svelte-test-ui/src/lib/components/Piece.svelte`, updating imports to use local types.

**Step 2: Create Square.svelte**

Copy from `apps/web/chess-devtools/svelte-test-ui/src/lib/components/Square.svelte`, updating to use props instead of store imports.

**Step 3: Create PromotionDialog.svelte**

Copy from `apps/web/chess-devtools/svelte-test-ui/src/lib/components/PromotionDialog.svelte`.

**Step 4: Create Board.svelte**

Refactor from original to use props instead of store imports:

```svelte
<!-- packages/board/src/lib/Board.svelte -->
<script lang="ts">
  import Square from './Square.svelte';
  import PromotionDialog from './PromotionDialog.svelte';
  import type { PieceInfo, Move, LastMove } from './types';

  interface Props {
    board: Map<string, PieceInfo>;
    legalMoves: Move[];
    flipped?: boolean;
    lastMove?: LastMove | null;
    check?: string | null;
    sideToMove?: 'white' | 'black';
    onMove?: (from: string, to: string, promotion?: string) => void;
  }

  let {
    board,
    legalMoves,
    flipped = false,
    lastMove = null,
    check = null,
    sideToMove = 'white',
    onMove,
  }: Props = $props();

  let selectedSquare: string | null = $state(null);
  let pendingPromotion: { from: string; to: string } | null = $state(null);

  const files = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];
  const ranks = ['8', '7', '6', '5', '4', '3', '2', '1'];

  const displayFiles = $derived(flipped ? [...files].reverse() : files);
  const displayRanks = $derived(flipped ? [...ranks].reverse() : ranks);

  function isLightSquare(file: string, rank: string): boolean {
    const fileIdx = files.indexOf(file);
    const rankIdx = parseInt(rank);
    return (fileIdx + rankIdx) % 2 === 1;
  }

  function getLegalMovesFrom(square: string): Move[] {
    return legalMoves.filter((m) => m.from === square);
  }

  function isLegalTarget(square: string): boolean {
    if (!selectedSquare) return false;
    return getLegalMovesFrom(selectedSquare).some((m) => m.to === square);
  }

  function isKingInCheck(square: string): boolean {
    return check === square;
  }

  function isPromotionMove(from: string, to: string): boolean {
    return getLegalMovesFrom(from)
      .filter((m) => m.to === to)
      .some((m) => m.promotion);
  }

  function executeMove(from: string, to: string, promotion?: string) {
    onMove?.(from, to, promotion);
    selectedSquare = null;
    pendingPromotion = null;
  }

  function handlePromotionSelect(piece: 'q' | 'r' | 'b' | 'n') {
    if (pendingPromotion) {
      executeMove(pendingPromotion.from, pendingPromotion.to, piece);
    }
  }

  function handlePromotionCancel() {
    pendingPromotion = null;
    selectedSquare = null;
  }

  function handleSquareClick(square: string) {
    const piece = board.get(square);

    if (selectedSquare && isLegalTarget(square)) {
      if (isPromotionMove(selectedSquare, square)) {
        pendingPromotion = { from: selectedSquare, to: square };
      } else {
        executeMove(selectedSquare, square);
      }
      return;
    }

    if (piece && piece.color === sideToMove) {
      selectedSquare = square;
      return;
    }

    selectedSquare = null;
  }

  function handleDragStart(e: DragEvent, square: string) {
    const piece = board.get(square);
    if (!piece || piece.color !== sideToMove) {
      e.preventDefault();
      return;
    }
    selectedSquare = square;
    e.dataTransfer!.effectAllowed = 'move';
    e.dataTransfer!.setData('text/plain', square);

    const img = (e.target as HTMLElement).querySelector('img');
    if (img) {
      const size = img.offsetWidth;
      e.dataTransfer!.setDragImage(img, size / 2, size / 2);
    }
  }

  function handleDragOver(e: DragEvent, square: string) {
    if (selectedSquare && isLegalTarget(square)) {
      e.preventDefault();
      e.dataTransfer!.dropEffect = 'move';
    }
  }

  function handleDrop(e: DragEvent, square: string) {
    e.preventDefault();
    if (selectedSquare && isLegalTarget(square)) {
      if (isPromotionMove(selectedSquare, square)) {
        pendingPromotion = { from: selectedSquare, to: square };
      } else {
        executeMove(selectedSquare, square);
      }
    } else {
      selectedSquare = null;
    }
  }

  $effect(() => {
    board;
    selectedSquare = null;
  });
</script>

<div class="board-container">
  <div class="board">
    {#each displayRanks as rank}
      {#each displayFiles as file}
        {@const square = file + rank}
        {@const piece = board.get(square) ?? null}
        <Square
          {square}
          {piece}
          isLight={isLightSquare(file, rank)}
          isSelected={selectedSquare === square}
          isLegalTarget={isLegalTarget(square)}
          isLastMove={lastMove?.from === square || lastMove?.to === square}
          isCheck={isKingInCheck(square)}
          onclick={() => handleSquareClick(square)}
          ondragstart={(e) => handleDragStart(e, square)}
          ondragover={(e) => handleDragOver(e, square)}
          ondrop={(e) => handleDrop(e, square)}
        />
      {/each}
    {/each}
  </div>
  <div class="coordinates files">
    {#each displayFiles as file}
      <span>{file}</span>
    {/each}
  </div>
  <div class="coordinates ranks">
    {#each displayRanks as rank}
      <span>{rank}</span>
    {/each}
  </div>
</div>

{#if pendingPromotion}
  <PromotionDialog
    color={sideToMove}
    onselect={handlePromotionSelect}
    oncancel={handlePromotionCancel}
  />
{/if}

<style>
  .board-container {
    position: relative;
    display: inline-block;
  }

  .board {
    display: grid;
    grid-template-columns: repeat(8, var(--square-size, 60px));
    grid-template-rows: repeat(8, var(--square-size, 60px));
    border: 2px solid var(--board-border, #333);
    border-radius: var(--board-radius, 4px);
    overflow: hidden;
  }

  .coordinates {
    position: absolute;
    display: grid;
    color: var(--coord-color, #666);
    font-size: 0.75rem;
    font-weight: 500;
  }

  .coordinates span {
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .coordinates.files {
    bottom: -1.5rem;
    left: 0;
    grid-template-columns: repeat(8, var(--square-size, 60px));
  }

  .coordinates.ranks {
    top: 0;
    left: -1.25rem;
    grid-template-rows: repeat(8, var(--square-size, 60px));
  }
</style>
```

**Step 5: Update index.ts**

```typescript
// packages/board/src/lib/index.ts

export * from './types';
export { default as Board } from './Board.svelte';
export { default as Square } from './Square.svelte';
export { default as Piece } from './Piece.svelte';
export { default as PromotionDialog } from './PromotionDialog.svelte';
```

**Step 6: Build**

Run: `cd packages/board && npm run build`
Expected: Build succeeds

**Step 7: Commit**

```bash
git add packages/board/
git commit -m "feat(board): port Board, Square, Piece, PromotionDialog components"
```

---

## Task 10: Add sounds helper to game-store

**Files:**
- Create: `packages/game-store/src/sounds.ts`
- Modify: `packages/game-store/src/index.ts`

**Step 1: Create sounds.ts**

```typescript
// packages/game-store/src/sounds.ts

import { writable } from 'svelte/store';
import type { SoundConfig } from './types';

const SOUND_BASE = 'https://lichess1.org/assets/sound/standard';

type SoundType = 'move' | 'capture' | 'check' | 'gameStart' | 'gameEnd';

const soundUrls: Record<SoundType, string> = {
  move: `${SOUND_BASE}/Move.mp3`,
  capture: `${SOUND_BASE}/Capture.mp3`,
  check: `${SOUND_BASE}/Check.mp3`,
  gameStart: `${SOUND_BASE}/GenericNotify.mp3`,
  gameEnd: `${SOUND_BASE}/Victory.mp3`,
};

/** Mute state store - persisted to localStorage if available */
export const isMuted = writable(false);

let currentMuted = false;
isMuted.subscribe((m) => (currentMuted = m));

/** Toggle mute state */
export function toggleMute(): void {
  isMuted.update((m) => !m);
}

const audioCache = new Map<SoundType, HTMLAudioElement>();

function getAudio(type: SoundType): HTMLAudioElement | null {
  if (typeof Audio === 'undefined') return null;

  let audio = audioCache.get(type);
  if (!audio) {
    audio = new Audio(soundUrls[type]);
    audio.volume = 0.5;
    audioCache.set(type, audio);
  }
  return audio;
}

function playSound(type: SoundType): void {
  if (currentMuted) return;
  const audio = getAudio(type);
  if (audio) {
    audio.currentTime = 0;
    audio.play().catch(() => {});
  }
}

/** Preload all sounds for instant playback */
export function preloadSounds(): void {
  Object.keys(soundUrls).forEach((type) => {
    getAudio(type as SoundType);
  });
}

/**
 * Creates a Lichess-style sound configuration.
 * Call preloadSounds() on init for best experience.
 */
export function lichessSounds(): SoundConfig {
  return {
    playMove: () => playSound('move'),
    playCapture: () => playSound('capture'),
    playCheck: () => playSound('check'),
    playGameStart: () => playSound('gameStart'),
    playGameEnd: () => playSound('gameEnd'),
  };
}
```

**Step 2: Update index.ts**

```typescript
// packages/game-store/src/index.ts

export * from './types';
export { createGameStore } from './store';
export { lichessSounds, preloadSounds, isMuted, toggleMute } from './sounds';
```

**Step 3: Build**

Run: `cd packages/game-store && npm run build`
Expected: Build succeeds

**Step 4: Commit**

```bash
git add packages/game-store/
git commit -m "feat(game-store): add lichessSounds helper and mute controls"
```

---

## Task 11: Update devtools app to use packages

**Files:**
- Modify: `apps/web/chess-devtools/svelte-test-ui/package.json`
- Modify: `apps/web/chess-devtools/svelte-test-ui/src/lib/stores/game.ts`
- Modify: `apps/web/chess-devtools/svelte-test-ui/src/lib/stores/bot.ts`
- Modify: `apps/web/chess-devtools/svelte-test-ui/src/lib/components/Board.svelte` (replace with import)

**Step 1: Add workspace dependencies to package.json**

```json
{
  "dependencies": {
    "@tmttn-chess/board": "workspace:*",
    "@tmttn-chess/game-store": "workspace:*",
    "@tmttn-chess/bot-client": "workspace:*"
  }
}
```

**Step 2: Refactor game.ts to use @tmttn-chess/game-store**

Replace the entire file to use createGameStore with WASM adapter.

**Step 3: Refactor bot.ts to use @tmttn-chess/bot-client**

Replace to use createBotClient and connect to the game store.

**Step 4: Update Board import in +page.svelte**

```svelte
<script>
  import { Board } from '@tmttn-chess/board';
  // ... rest unchanged, just pass props instead of relying on stores
</script>
```

**Step 5: Run npm install from root**

Run: `npm install`
Expected: Workspace packages are linked

**Step 6: Build and test devtools**

Run: `cd apps/web/chess-devtools/svelte-test-ui && npm run build`
Expected: Build succeeds

**Step 7: Commit**

```bash
git add apps/web/chess-devtools/svelte-test-ui/
git commit -m "refactor(devtools): migrate to @tmttn-chess packages"
```

---

## Task 12: Add package READMEs

**Files:**
- Create: `packages/game-store/README.md`
- Create: `packages/bot-client/README.md`
- Create: `packages/board/README.md`

**Step 1: Create game-store README**

Document API, installation, and usage examples.

**Step 2: Create bot-client README**

Document API, callbacks, and session management.

**Step 3: Create board README**

Document components, props, and CSS custom properties for theming.

**Step 4: Commit**

```bash
git add packages/*/README.md
git commit -m "docs: add README files for all packages"
```

---

## Task 13: Setup changesets for versioning

**Files:**
- Create: `.changeset/config.json`
- Modify: `package.json` (add changeset scripts)

**Step 1: Install changesets**

Run: `npm install -D @changesets/cli`

**Step 2: Initialize changesets**

Run: `npx changeset init`

**Step 3: Configure for npm workspaces**

Update `.changeset/config.json`:

```json
{
  "$schema": "https://unpkg.com/@changesets/config@3.0.0/schema.json",
  "changelog": "@changesets/cli/changelog",
  "commit": false,
  "fixed": [],
  "linked": [],
  "access": "public",
  "baseBranch": "main",
  "updateInternalDependencies": "patch",
  "ignore": []
}
```

**Step 4: Add scripts to root package.json**

```json
{
  "scripts": {
    "changeset": "changeset",
    "version": "changeset version",
    "publish": "npm run build && changeset publish"
  }
}
```

**Step 5: Commit**

```bash
git add .changeset/ package.json
git commit -m "chore: setup changesets for package versioning"
```

---

## Task 14: Final verification

**Step 1: Build all packages**

Run: `npm run build`
Expected: All packages build successfully

**Step 2: Run all tests**

Run: `npm run test`
Expected: All tests pass

**Step 3: Verify devtools works**

Run: `cd apps/web/chess-devtools/svelte-test-ui && npm run dev`
Expected: App runs and functions correctly

**Step 4: Test publish dry-run**

Run: `cd packages/game-store && npm publish --dry-run`
Expected: Shows what would be published

**Step 5: Create final commit**

```bash
git add -A
git commit -m "feat: complete Phase 4 shared libraries extraction"
```

---

*Last updated: 2025-01-20*
