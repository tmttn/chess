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
