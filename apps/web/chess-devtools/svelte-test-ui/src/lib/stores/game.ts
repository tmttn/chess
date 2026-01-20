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

export interface MoveSearchInfo {
  depth: number;
  score: number;
  nodes: number;
  time: number;
  pv: string[];
}

export interface MoveHistoryEntry {
  uci: string;
  san: string;
  fen: string;
  searchInfo?: MoveSearchInfo; // Optional bot search info for this move
}

export interface GameState {
  game: Game | null;
  fen: string;                // Live position FEN
  legalMoves: Move[];         // Legal moves at live position
  board: Map<string, PieceInfo>;  // Board at viewed position (for display)
  liveBoard: Map<string, PieceInfo>; // Board at live position
  moveHistory: MoveHistoryEntry[];
  viewIndex: number;          // -1 = start position, 0+ = after that move
  isCheck: boolean;           // At live position
  isGameOver: boolean;        // At live position
  result: string | null;      // At live position
  sideToMove: 'white' | 'black'; // At live position
}

const STARTING_FEN = 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1';

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
  sideToMove: 'white'
};

function createGameStore() {
  const { subscribe, set, update } = writable<GameState>(initialState);

  function refreshState(game: Game): Partial<GameState> {
    const board = getBoardState(game);
    return {
      fen: game.toFen(),
      legalMoves: getLegalMoves(game),
      board: board,
      liveBoard: board,
      isCheck: game.isCheck(),
      isGameOver: game.isGameOver(),
      result: game.result() ?? null,
      sideToMove: game.sideToMove() as 'white' | 'black'
    };
  }

  // Get board state for a given FEN (for viewing history)
  function getBoardForFen(fen: string): Map<string, PieceInfo> {
    const tempGame = loadFen(fen);
    if (!tempGame) return new Map();
    return getBoardState(tempGame);
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

        // Check if this is a capture (piece on destination square at live position)
        const move = parseUci(uci);
        const isCapture = state.liveBoard.has(move.to);

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

          // Always append to end of history (never truncate)
          const newHistory = [...state.moveHistory, { uci, san, fen: newState.fen! }];
          return {
            ...state,
            ...newState,
            moveHistory: newHistory,
            viewIndex: newHistory.length - 1  // Jump to live position
          };
        }
        return state;
      });
    },

    // Navigate view backwards (doesn't affect live game state)
    viewPrev() {
      update(state => {
        if (state.viewIndex < 0) return state;

        const newIndex = state.viewIndex - 1;
        const fenToView = newIndex >= 0
          ? state.moveHistory[newIndex]!.fen
          : STARTING_FEN;

        return {
          ...state,
          board: getBoardForFen(fenToView),
          viewIndex: newIndex
        };
      });
    },

    // Navigate view forwards (doesn't affect live game state)
    viewNext() {
      update(state => {
        if (state.viewIndex >= state.moveHistory.length - 1) return state;

        const newIndex = state.viewIndex + 1;
        const fenToView = state.moveHistory[newIndex]!.fen;

        return {
          ...state,
          board: getBoardForFen(fenToView),
          viewIndex: newIndex
        };
      });
    },

    // Go to specific move in history (view only)
    goToMove(index: number) {
      update(state => {
        if (index < -1 || index >= state.moveHistory.length) return state;

        const fenToView = index >= 0
          ? state.moveHistory[index]!.fen
          : STARTING_FEN;

        return {
          ...state,
          board: getBoardForFen(fenToView),
          viewIndex: index
        };
      });
    },

    // Go to start of game (view only)
    goToStart() {
      update(state => {
        return {
          ...state,
          board: getBoardForFen(STARTING_FEN),
          viewIndex: -1
        };
      });
    },

    // Go to live position (latest move)
    goToLive() {
      update(state => {
        return {
          ...state,
          board: state.liveBoard,
          viewIndex: state.moveHistory.length - 1
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
    },

    // Attach search info to the last move (called by bot store after bestmove)
    attachSearchInfoToLastMove(searchInfo: MoveSearchInfo) {
      update(state => {
        if (state.moveHistory.length === 0) return state;

        const newHistory = [...state.moveHistory];
        const lastIndex = newHistory.length - 1;
        newHistory[lastIndex] = {
          ...newHistory[lastIndex],
          searchInfo
        };

        return { ...state, moveHistory: newHistory };
      });
    }
  };
}

export const gameStore = createGameStore();

const STARTING_FEN_CONST = 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1';

// Derived stores for convenience
export const board: Readable<Map<string, PieceInfo>> = derived(gameStore, $game => $game.board);
export const legalMoves: Readable<Move[]> = derived(gameStore, $game => $game.legalMoves);
export const sideToMove: Readable<'white' | 'black'> = derived(gameStore, $game => $game.sideToMove);
export const isCheck: Readable<boolean> = derived(gameStore, $game => $game.isCheck);
export const isGameOver: Readable<boolean> = derived(gameStore, $game => $game.isGameOver);
export const viewIndex: Readable<number> = derived(gameStore, $game => $game.viewIndex);
export const isViewingHistory: Readable<boolean> = derived(
  gameStore,
  $game => $game.viewIndex < $game.moveHistory.length - 1
);
export const moveCount: Readable<number> = derived(gameStore, $game => $game.moveHistory.length);

// Viewed position info (for history browsing)
export const viewFen: Readable<string> = derived(
  gameStore,
  $game => $game.viewIndex >= 0
    ? $game.moveHistory[$game.viewIndex]?.fen ?? STARTING_FEN_CONST
    : STARTING_FEN_CONST
);

// Live position FEN
export const liveFen: Readable<string> = derived(gameStore, $game => $game.fen);

// Search info for viewed move (for history browsing)
export const viewSearchInfo: Readable<MoveSearchInfo | null> = derived(
  gameStore,
  $game => $game.viewIndex >= 0
    ? $game.moveHistory[$game.viewIndex]?.searchInfo ?? null
    : null
);
