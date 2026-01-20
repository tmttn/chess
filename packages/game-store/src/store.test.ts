// packages/game-store/src/store.test.ts

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import { createGameStore } from './store';
import type { GameAdapter, GameStoreConfig, Move, PieceInfo } from './types';

const STARTING_FEN = 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1';

// Mock game adapter factory
function createMockGame(fen = STARTING_FEN): GameAdapter {
  let currentFen = fen;
  return {
    toFen: () => currentFen,
    isCheck: () => false,
    isGameOver: () => false,
    result: () => null,
    sideToMove: () => 'white',
  };
}

// Mock config factory
function createMockConfig(): GameStoreConfig {
  const initialBoard = new Map<string, PieceInfo>([['e2', { type: 'p', color: 'white' }]]);

  return {
    createGame: vi.fn(() => createMockGame()),
    loadFen: vi.fn((fen) => createMockGame(fen)),
    getLegalMoves: vi.fn(() => [
      { from: 'e2', to: 'e4', uci: 'e2e4' },
      { from: 'e2', to: 'e3', uci: 'e2e3' },
    ]),
    makeMove: vi.fn(() => true),
    getBoardState: vi.fn(() => new Map(initialBoard)),
    moveToSan: vi.fn(() => 'e4'),
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

  it('should go to a specific move index', () => {
    const store = createGameStore(config);
    store.init();
    store.makeMove('e2e4');
    store.makeMove('e7e5');
    store.makeMove('d2d4');

    expect(get(store.viewIndex)).toBe(2);

    store.goToMove(0);
    expect(get(store.viewIndex)).toBe(0);

    store.goToMove(2);
    expect(get(store.viewIndex)).toBe(2);

    store.goToMove(-1);
    expect(get(store.viewIndex)).toBe(-1);
  });

  it('should not navigate outside valid bounds', () => {
    const store = createGameStore(config);
    store.init();
    store.makeMove('e2e4');

    // Try to go before start
    store.goToStart();
    store.viewPrev();
    expect(get(store.viewIndex)).toBe(-1);

    // Try to go after end
    store.goToLive();
    store.viewNext();
    expect(get(store.viewIndex)).toBe(0);

    // Invalid goToMove indices
    store.goToMove(100);
    expect(get(store.viewIndex)).toBe(0);

    store.goToMove(-5);
    expect(get(store.viewIndex)).toBe(0);
  });

  it('should load a FEN position', () => {
    const store = createGameStore(config);
    const customFen = 'r1bqkbnr/pppppppp/2n5/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 1 2';

    const result = store.loadFen(customFen);

    expect(result).toBe(true);
    expect(config.loadFen).toHaveBeenCalledWith(customFen);
    expect(get(store.moveHistory).length).toBe(0);
  });

  it('should return false when loading invalid FEN', () => {
    const configWithInvalidFen = createMockConfig();
    configWithInvalidFen.loadFen = vi.fn(() => null);

    const store = createGameStore(configWithInvalidFen);
    const result = store.loadFen('invalid-fen');

    expect(result).toBe(false);
  });

  it('should not make move before initialization', () => {
    const store = createGameStore(config);
    // No init() called
    const result = store.makeMove('e2e4');

    expect(result).toBe(false);
    expect(get(store.moveHistory).length).toBe(0);
  });

  it('should not update history when makeMove fails', () => {
    const configWithFailingMove = createMockConfig();
    configWithFailingMove.makeMove = vi.fn(() => false);

    const store = createGameStore(configWithFailingMove);
    store.init();

    const result = store.makeMove('e2e4');

    expect(result).toBe(false);
    expect(get(store.moveHistory).length).toBe(0);
  });

  it('should play capture sound when capturing', () => {
    // Setup config where destination square has a piece
    const configWithCapture = createMockConfig();
    configWithCapture.getBoardState = vi.fn(
      () =>
        new Map<string, PieceInfo>([
          ['e2', { type: 'p', color: 'white' }],
          ['e4', { type: 'p', color: 'black' }], // Target square has a piece
        ])
    );

    const store = createGameStore(configWithCapture);
    store.init();
    store.makeMove('e2e4');

    expect(configWithCapture.sounds?.playCapture).toHaveBeenCalled();
  });

  it('should play check sound when in check', () => {
    let isCheck = false;
    const gameWithCheck: GameAdapter = {
      toFen: () => STARTING_FEN,
      isCheck: () => isCheck,
      isGameOver: () => false,
      result: () => null,
      sideToMove: () => 'white',
    };

    const configWithCheck = createMockConfig();
    configWithCheck.createGame = vi.fn(() => gameWithCheck);
    configWithCheck.loadFen = vi.fn(() => gameWithCheck);
    configWithCheck.makeMove = vi.fn(() => {
      isCheck = true;
      return true;
    });

    const store = createGameStore(configWithCheck);
    store.init();
    store.makeMove('e2e4');

    expect(configWithCheck.sounds?.playCheck).toHaveBeenCalled();
  });

  it('should play game end sound when game is over', () => {
    let isGameOver = false;
    const gameWithEnd: GameAdapter = {
      toFen: () => STARTING_FEN,
      isCheck: () => false,
      isGameOver: () => isGameOver,
      result: () => (isGameOver ? '1-0' : null),
      sideToMove: () => 'white',
    };

    const configWithEnd = createMockConfig();
    configWithEnd.createGame = vi.fn(() => gameWithEnd);
    configWithEnd.loadFen = vi.fn(() => gameWithEnd);
    configWithEnd.makeMove = vi.fn(() => {
      isGameOver = true;
      return true;
    });

    const store = createGameStore(configWithEnd);
    store.init();
    store.makeMove('e2e4');

    expect(configWithEnd.sounds?.playGameEnd).toHaveBeenCalled();
  });

  it('should not attach search info when no moves exist', () => {
    const store = createGameStore(config);
    store.init();

    const searchInfo = { depth: 20, score: 35, nodes: 1000, time: 500, pv: ['e2e4'] };
    store.attachSearchInfoToLastMove(searchInfo);

    expect(get(store.moveHistory).length).toBe(0);
  });

  it('should provide correct derived store values', () => {
    const store = createGameStore(config);
    store.init();

    expect(get(store.sideToMove)).toBe('white');
    expect(get(store.isCheck)).toBe(false);
    expect(get(store.isGameOver)).toBe(false);
    expect(get(store.liveFen)).toBe(STARTING_FEN);
    expect(get(store.viewFen)).toBe(STARTING_FEN);
    expect(get(store.viewSearchInfo)).toBe(null);
  });

  it('should update viewFen when navigating history', () => {
    const moveFen = 'rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1';

    const configWithFen = createMockConfig();
    let currentFen = STARTING_FEN;

    configWithFen.createGame = vi.fn(() => ({
      toFen: () => currentFen,
      isCheck: () => false,
      isGameOver: () => false,
      result: () => null,
      sideToMove: () => 'white',
    }));

    configWithFen.makeMove = vi.fn(() => {
      currentFen = moveFen;
      return true;
    });

    const store = createGameStore(configWithFen);
    store.init();
    store.makeMove('e2e4');

    // viewFen should match the current move's FEN
    expect(get(store.viewFen)).toBe(moveFen);

    store.goToStart();
    expect(get(store.viewFen)).toBe(STARTING_FEN);
  });

  it('should provide viewSearchInfo for current view position', () => {
    const store = createGameStore(config);
    store.init();
    store.makeMove('e2e4');
    store.makeMove('e7e5');

    const searchInfo1 = { depth: 20, score: 35, nodes: 1000, time: 500, pv: ['e2e4'] };
    store.goToMove(0);
    // Can't attach to non-last move in current implementation
    // So we test with the last move
    store.goToLive();
    store.attachSearchInfoToLastMove(searchInfo1);

    expect(get(store.viewSearchInfo)).toEqual(searchInfo1);

    store.goToMove(0);
    expect(get(store.viewSearchInfo)).toBe(null);
  });

  it('should work without sounds config', () => {
    const configWithoutSounds: GameStoreConfig = {
      createGame: () => createMockGame(),
      loadFen: (fen) => createMockGame(fen),
      getLegalMoves: () => [{ from: 'e2', to: 'e4', uci: 'e2e4' }],
      makeMove: () => true,
      getBoardState: () => new Map([['e2', { type: 'p', color: 'white' }]]),
    };

    const store = createGameStore(configWithoutSounds);

    // Should not throw
    expect(() => store.init()).not.toThrow();
    expect(() => store.makeMove('e2e4')).not.toThrow();
    expect(() => store.newGame()).not.toThrow();
  });

  it('should use default parseUci when not provided', () => {
    const configWithoutParseUci: GameStoreConfig = {
      createGame: () => createMockGame(),
      loadFen: (fen) => createMockGame(fen),
      getLegalMoves: () => [{ from: 'e2', to: 'e4', uci: 'e2e4' }],
      makeMove: () => true,
      getBoardState: () => new Map([['e2', { type: 'p', color: 'white' }]]),
      moveToSan: () => 'e4',
    };

    const store = createGameStore(configWithoutParseUci);
    store.init();

    // Should work with default parsing
    const result = store.makeMove('e2e4');
    expect(result).toBe(true);
  });

  it('should play game start sound on newGame()', () => {
    const store = createGameStore(config);
    store.init();
    store.makeMove('e2e4');

    vi.clearAllMocks();
    store.newGame();

    expect(config.sounds?.playGameStart).toHaveBeenCalled();
  });
});
