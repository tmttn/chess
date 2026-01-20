import { writable } from 'svelte/store';
import { browser } from '$app/environment';

export const wasmReady = writable(false);

// Import the chess-wasm module statically so Vite can resolve it
import * as chessWasmModule from 'chess-wasm';

let initialized = false;
let GameClass: typeof chessWasmModule.Game | null = null;

export async function initWasm(): Promise<void> {
  if (!browser) return;
  if (initialized) return;

  try {
    // Use the WASM file from static folder
    await chessWasmModule.default('/chess_wasm_bg.wasm');

    GameClass = chessWasmModule.Game;
    initialized = true;
    wasmReady.set(true);
    console.log('WASM initialized successfully');
  } catch (error) {
    console.error('Failed to initialize WASM:', error);
    throw error;
  }
}

export interface Move {
  uci: string;
  from: string;
  to: string;
  promotion?: string;
}

export interface PieceInfo {
  type: 'p' | 'n' | 'b' | 'r' | 'q' | 'k';
  color: 'white' | 'black';
}

export function parseUci(uci: string): Move {
  const from = uci.slice(0, 2);
  const to = uci.slice(2, 4);
  const promotion = uci.length > 4 ? uci[4] : undefined;
  return { uci, from, to, promotion };
}

export function parsePiece(fenChar: string): PieceInfo {
  const lower = fenChar.toLowerCase();
  const color = fenChar === lower ? 'black' : 'white';
  const typeMap: Record<string, PieceInfo['type']> = {
    'p': 'p', 'n': 'n', 'b': 'b', 'r': 'r', 'q': 'q', 'k': 'k'
  };
  return { type: typeMap[lower]!, color };
}

export type Game = typeof chessWasmModule.Game extends new (...args: any[]) => infer R ? R : never;

export function createGame(): Game {
  if (!GameClass) throw new Error('WASM not initialized');
  return new GameClass();
}

export function loadFen(fen: string): Game | null {
  if (!GameClass) throw new Error('WASM not initialized');
  try {
    return GameClass.fromFen(fen);
  } catch {
    return null;
  }
}

export function getLegalMoves(game: Game): Move[] {
  return game.legalMoves().map(parseUci);
}

export function makeMove(game: Game, uci: string): boolean {
  try {
    game.makeMove(uci);
    return true;
  } catch {
    return false;
  }
}

export function getBoardState(game: Game): Map<string, PieceInfo> {
  const board = new Map<string, PieceInfo>();
  const files = 'abcdefgh';
  const ranks = '12345678';

  for (const file of files) {
    for (const rank of ranks) {
      const square = file + rank;
      const piece = game.pieceAt(square);
      if (piece) {
        board.set(square, parsePiece(piece));
      }
    }
  }

  return board;
}
