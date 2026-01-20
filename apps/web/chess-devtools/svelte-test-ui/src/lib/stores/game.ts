// apps/web/chess-devtools/svelte-test-ui/src/lib/stores/game.ts
//
// Game store using @tmttn-chess/game-store with WASM adapter

import {
  createGameStore,
  lichessSounds,
  preloadSounds,
  type GameStore,
  type GameStoreConfig,
  type GameAdapter,
  type Move,
  type PieceInfo,
  type MoveSearchInfo,
  type MoveHistoryEntry,
} from '@tmttn-chess/game-store';
import {
  createGame as wasmCreateGame,
  loadFen as wasmLoadFen,
  getLegalMoves as wasmGetLegalMoves,
  makeMove as wasmMakeMove,
  moveToSan as wasmMoveToSan,
  getBoardState as wasmGetBoardState,
  parseUci,
  type Game,
} from '../wasm';

// Re-export types for consumers
export type { MoveSearchInfo, MoveHistoryEntry, Move, PieceInfo };

/**
 * Wraps a WASM Game object to implement the GameAdapter interface.
 */
function createWasmAdapter(game: Game): GameAdapter {
  return {
    toFen: () => game.toFen(),
    isCheck: () => game.isCheck(),
    isGameOver: () => game.isGameOver(),
    result: () => game.result() ?? null,
    sideToMove: () => game.sideToMove(),
  };
}

// Store the raw WASM games so we can operate on them
const gameMap = new WeakMap<GameAdapter, Game>();

/**
 * Create a game and return an adapter, while storing the raw game.
 */
function createGameWithAdapter(): GameAdapter {
  const game = wasmCreateGame();
  const adapter = createWasmAdapter(game);
  gameMap.set(adapter, game);
  return adapter;
}

/**
 * Load a FEN and return an adapter, while storing the raw game.
 */
function loadFenWithAdapter(fen: string): GameAdapter | null {
  const game = wasmLoadFen(fen);
  if (!game) return null;
  const adapter = createWasmAdapter(game);
  gameMap.set(adapter, game);
  return adapter;
}

/**
 * Get the raw WASM game from an adapter.
 */
function getWasmGame(adapter: GameAdapter): Game {
  const game = gameMap.get(adapter);
  if (!game) throw new Error('Game not found in map');
  return game;
}

/**
 * Configuration for the game store using WASM functions.
 */
const gameStoreConfig: GameStoreConfig = {
  createGame: createGameWithAdapter,
  loadFen: loadFenWithAdapter,
  getLegalMoves: (adapter) => wasmGetLegalMoves(getWasmGame(adapter)),
  makeMove: (adapter, uci) => wasmMakeMove(getWasmGame(adapter), uci),
  getBoardState: (adapter) => wasmGetBoardState(getWasmGame(adapter)),
  moveToSan: (adapter, uci) => wasmMoveToSan(getWasmGame(adapter), uci),
  parseUci: (uci) => {
    const parsed = parseUci(uci);
    return { from: parsed.from, to: parsed.to };
  },
  sounds: lichessSounds(),
};

// Create the game store instance
const store = createGameStore(gameStoreConfig);

// Wrap init to also preload sounds
const originalInit = store.init.bind(store);
const gameStore: GameStore = {
  ...store,
  init() {
    preloadSounds();
    originalInit();
  },
};

export { gameStore };

// Re-export derived stores for convenience
export const board = gameStore.board;
export const legalMoves = gameStore.legalMoves;
export const sideToMove = gameStore.sideToMove;
export const isCheck = gameStore.isCheck;
export const isGameOver = gameStore.isGameOver;
export const viewIndex = gameStore.viewIndex;
export const isViewingHistory = gameStore.isViewingHistory;
export const moveHistory = gameStore.moveHistory;
export const viewFen = gameStore.viewFen;
export const liveFen = gameStore.liveFen;
export const viewSearchInfo = gameStore.viewSearchInfo;
