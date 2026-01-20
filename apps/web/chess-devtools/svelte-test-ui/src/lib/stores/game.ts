import { writable, derived, type Readable } from 'svelte/store';
import {
  createGame,
  loadFen,
  getLegalMoves,
  makeMove as wasmMakeMove,
  moveToSan,
  getBoardState,
  parseUci,
  type Move,
  type PieceInfo,
  type Game
} from '../wasm';
import { playSound, preloadSounds } from '../sounds';

export interface GameState {
  game: Game | null;
  fen: string;
  legalMoves: Move[];
  board: Map<string, PieceInfo>;
  moveHistory: { uci: string; san: string; fen: string }[];
  viewIndex: number; // -1 = start position, 0+ = after that move
  isCheck: boolean;
  isGameOver: boolean;
  result: string | null;
  sideToMove: 'white' | 'black';
}

const STARTING_FEN = 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1';

const initialState: GameState = {
  game: null,
  fen: STARTING_FEN,
  legalMoves: [],
  board: new Map(),
  moveHistory: [],
  viewIndex: -1,
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
      preloadSounds();
      const game = createGame();
      set({
        ...initialState,
        game,
        ...refreshState(game)
      });
      playSound('gameStart');
    },

    makeMove(uci: string) {
      update(state => {
        if (!state.game) return state;

        // Check if this is a capture (piece on destination square)
        const move = parseUci(uci);
        const isCapture = state.board.has(move.to);

        // Get SAN before making the move (needs current position)
        const san = moveToSan(state.game, uci) ?? uci;

        const success = wasmMakeMove(state.game, uci);
        if (success) {
          const newState = refreshState(state.game);

          // Play appropriate sound
          if (newState.isGameOver) {
            playSound('gameEnd');
          } else if (newState.isCheck) {
            playSound('check');
          } else if (isCapture) {
            playSound('capture');
          } else {
            playSound('move');
          }

          const newHistory = [...state.moveHistory.slice(0, state.viewIndex + 1), { uci, san, fen: newState.fen! }];
          return {
            ...state,
            ...newState,
            moveHistory: newHistory,
            viewIndex: newHistory.length - 1
          };
        }
        return state;
      });
    },

    undo() {
      update(state => {
        if (!state.game || state.viewIndex < 0) return state;

        const newIndex = state.viewIndex - 1;
        const fenToLoad = newIndex >= 0
          ? state.moveHistory[newIndex]!.fen
          : STARTING_FEN;

        const newGame = loadFen(fenToLoad);
        if (!newGame) return state;

        return {
          ...state,
          game: newGame,
          ...refreshState(newGame),
          viewIndex: newIndex
        };
      });
    },

    redo() {
      update(state => {
        if (!state.game || state.viewIndex >= state.moveHistory.length - 1) return state;

        const newIndex = state.viewIndex + 1;
        const fenToLoad = state.moveHistory[newIndex]!.fen;

        const newGame = loadFen(fenToLoad);
        if (!newGame) return state;

        return {
          ...state,
          game: newGame,
          ...refreshState(newGame),
          viewIndex: newIndex
        };
      });
    },

    goToMove(index: number) {
      update(state => {
        if (!state.game) return state;
        if (index < -1 || index >= state.moveHistory.length) return state;

        const fenToLoad = index >= 0
          ? state.moveHistory[index]!.fen
          : STARTING_FEN;

        const newGame = loadFen(fenToLoad);
        if (!newGame) return state;

        return {
          ...state,
          game: newGame,
          ...refreshState(newGame),
          viewIndex: index
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
      playSound('gameStart');
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
export const viewIndex: Readable<number> = derived(gameStore, $game => $game.viewIndex);
