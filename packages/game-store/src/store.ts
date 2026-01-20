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
  const parseUci =
    config.parseUci ??
    ((uci: string) => ({
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
