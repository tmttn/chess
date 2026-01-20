import { writable, derived, type Readable } from 'svelte/store';
import {
  createGame,
  loadFen,
  getLegalMoves,
  makeMove as wasmMakeMove,
  getBoardState,
  type Move,
  type PieceInfo,
  type Game
} from '../wasm';

export interface GameState {
  game: Game | null;
  fen: string;
  legalMoves: Move[];
  board: Map<string, PieceInfo>;
  moveHistory: { uci: string; fen: string }[];
  isCheck: boolean;
  isGameOver: boolean;
  result: string | null;
  sideToMove: 'white' | 'black';
}

const initialState: GameState = {
  game: null,
  fen: 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1',
  legalMoves: [],
  board: new Map(),
  moveHistory: [],
  isCheck: false,
  isGameOver: false,
  result: null,
  sideToMove: 'white'
};

function createGameStore() {
  const { subscribe, set, update } = writable<GameState>(initialState);

  function refreshState(game: Game): Partial<GameState> {
    return {
      fen: game.toFen(),
      legalMoves: getLegalMoves(game),
      board: getBoardState(game),
      isCheck: game.isCheck(),
      isGameOver: game.isGameOver(),
      result: game.result() ?? null,
      sideToMove: game.sideToMove() as 'white' | 'black'
    };
  }

  return {
    subscribe,

    init() {
      const game = createGame();
      set({
        ...initialState,
        game,
        ...refreshState(game)
      });
    },

    makeMove(uci: string) {
      update(state => {
        if (!state.game) return state;
        const success = wasmMakeMove(state.game, uci);
        if (success) {
          const newState = refreshState(state.game);
          return {
            ...state,
            ...newState,
            moveHistory: [...state.moveHistory, { uci, fen: newState.fen! }]
          };
        }
        return state;
      });
    },

    undo() {
      update(state => {
        if (!state.game || state.moveHistory.length === 0) return state;

        const newHistory = state.moveHistory.slice(0, -1);
        const fenToLoad = newHistory.length > 0
          ? newHistory[newHistory.length - 1]!.fen
          : 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1';

        const newGame = loadFen(fenToLoad);
        if (!newGame) return state;

        return {
          ...state,
          game: newGame,
          ...refreshState(newGame),
          moveHistory: newHistory
        };
      });
    },

    newGame() {
      const game = createGame();
      set({
        ...initialState,
        game,
        ...refreshState(game)
      });
    },

    loadFen(fen: string): boolean {
      const newGame = loadFen(fen);
      if (!newGame) return false;

      set({
        ...initialState,
        game: newGame,
        ...refreshState(newGame)
      });
      return true;
    }
  };
}

export const gameStore = createGameStore();

// Derived stores for convenience
export const board: Readable<Map<string, PieceInfo>> = derived(gameStore, $game => $game.board);
export const legalMoves: Readable<Move[]> = derived(gameStore, $game => $game.legalMoves);
export const sideToMove: Readable<'white' | 'black'> = derived(gameStore, $game => $game.sideToMove);
export const isCheck: Readable<boolean> = derived(gameStore, $game => $game.isCheck);
export const isGameOver: Readable<boolean> = derived(gameStore, $game => $game.isGameOver);
