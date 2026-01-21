// FEN parsing utility tests
import { describe, it, expect } from 'vitest';
import { parseFen, getSideToMove, parseUciMove, findKing, STARTING_FEN } from './fen';

describe('parseFen', () => {
  it('parses starting position correctly', () => {
    const board = parseFen(STARTING_FEN);

    // Check white pieces on rank 1
    expect(board.get('a1')).toEqual({ type: 'r', color: 'white' });
    expect(board.get('b1')).toEqual({ type: 'n', color: 'white' });
    expect(board.get('c1')).toEqual({ type: 'b', color: 'white' });
    expect(board.get('d1')).toEqual({ type: 'q', color: 'white' });
    expect(board.get('e1')).toEqual({ type: 'k', color: 'white' });
    expect(board.get('f1')).toEqual({ type: 'b', color: 'white' });
    expect(board.get('g1')).toEqual({ type: 'n', color: 'white' });
    expect(board.get('h1')).toEqual({ type: 'r', color: 'white' });

    // Check white pawns on rank 2
    for (const file of 'abcdefgh') {
      expect(board.get(`${file}2`)).toEqual({ type: 'p', color: 'white' });
    }

    // Check black pieces on rank 8
    expect(board.get('a8')).toEqual({ type: 'r', color: 'black' });
    expect(board.get('e8')).toEqual({ type: 'k', color: 'black' });

    // Check black pawns on rank 7
    for (const file of 'abcdefgh') {
      expect(board.get(`${file}7`)).toEqual({ type: 'p', color: 'black' });
    }

    // Check empty squares
    expect(board.get('e4')).toBeUndefined();
    expect(board.get('d5')).toBeUndefined();

    // Should have 32 pieces total
    expect(board.size).toBe(32);
  });

  it('parses position-only FEN (no additional fields)', () => {
    const board = parseFen('8/8/8/8/4K3/8/8/8');

    expect(board.size).toBe(1);
    expect(board.get('e4')).toEqual({ type: 'k', color: 'white' });
  });

  it('handles empty board', () => {
    const board = parseFen('8/8/8/8/8/8/8/8 w - - 0 1');

    expect(board.size).toBe(0);
  });

  it('parses position after 1.e4 e5', () => {
    const fen = 'rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq e6 0 2';
    const board = parseFen(fen);

    // e4 pawn
    expect(board.get('e4')).toEqual({ type: 'p', color: 'white' });
    // e5 pawn
    expect(board.get('e5')).toEqual({ type: 'p', color: 'black' });
    // e2 should be empty
    expect(board.get('e2')).toBeUndefined();
    // e7 should be empty
    expect(board.get('e7')).toBeUndefined();
  });
});

describe('getSideToMove', () => {
  it('returns white for starting position', () => {
    expect(getSideToMove(STARTING_FEN)).toBe('white');
  });

  it('returns black when side to move is b', () => {
    const fen = 'rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1';
    expect(getSideToMove(fen)).toBe('black');
  });

  it('returns white when side to move is w', () => {
    const fen = 'rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq e6 0 2';
    expect(getSideToMove(fen)).toBe('white');
  });

  it('defaults to white for malformed FEN', () => {
    expect(getSideToMove('8/8/8/8/8/8/8/8')).toBe('white');
  });
});

describe('parseUciMove', () => {
  it('parses standard move', () => {
    const move = parseUciMove('e2e4');
    expect(move).toEqual({ from: 'e2', to: 'e4' });
  });

  it('parses promotion move', () => {
    const move = parseUciMove('e7e8q');
    expect(move).toEqual({ from: 'e7', to: 'e8' });
  });

  it('parses castling move', () => {
    const kingSide = parseUciMove('e1g1');
    expect(kingSide).toEqual({ from: 'e1', to: 'g1' });

    const queenSide = parseUciMove('e1c1');
    expect(queenSide).toEqual({ from: 'e1', to: 'c1' });
  });
});

describe('findKing', () => {
  it('finds white king in starting position', () => {
    const board = parseFen(STARTING_FEN);
    expect(findKing(board, 'white')).toBe('e1');
  });

  it('finds black king in starting position', () => {
    const board = parseFen(STARTING_FEN);
    expect(findKing(board, 'black')).toBe('e8');
  });

  it('finds king after it has moved', () => {
    // King has castled kingside
    const fen = 'r1bqk2r/pppp1ppp/2n2n2/2b1p3/2B1P3/5N2/PPPP1PPP/RNBQ1RK1 b kq - 5 4';
    const board = parseFen(fen);
    expect(findKing(board, 'white')).toBe('g1');
    expect(findKing(board, 'black')).toBe('e8');
  });

  it('returns null if king not found', () => {
    const board = new Map();
    expect(findKing(board, 'white')).toBeNull();
    expect(findKing(board, 'black')).toBeNull();
  });
});

describe('STARTING_FEN', () => {
  it('is a valid FEN string', () => {
    expect(STARTING_FEN).toBe('rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1');
  });

  it('produces correct board when parsed', () => {
    const board = parseFen(STARTING_FEN);
    expect(board.size).toBe(32);
  });
});
