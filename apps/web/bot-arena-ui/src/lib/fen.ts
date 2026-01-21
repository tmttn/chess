// FEN (Forsyth-Edwards Notation) parser utilities
import type { PieceInfo, LastMove } from '@tmttn-chess/board';

/** Piece type mapping from FEN character */
const PIECE_MAP: Record<string, PieceInfo['type']> = {
  p: 'p',
  n: 'n',
  b: 'b',
  r: 'r',
  q: 'q',
  k: 'k',
};

/**
 * Parse a FEN string into a board Map
 * @param fen - FEN string (full or position-only)
 * @returns Board map with piece information for each occupied square
 */
export function parseFen(fen: string): Map<string, PieceInfo> {
  const board = new Map<string, PieceInfo>();
  const position = fen.split(' ')[0];
  const ranks = position.split('/');

  for (let rankIdx = 0; rankIdx < 8; rankIdx++) {
    const rank = ranks[rankIdx];
    let fileIdx = 0;

    for (const char of rank) {
      if (char >= '1' && char <= '8') {
        // Empty squares
        fileIdx += parseInt(char);
      } else {
        const file = String.fromCharCode('a'.charCodeAt(0) + fileIdx);
        const rankNum = 8 - rankIdx;
        const square = `${file}${rankNum}`;

        const pieceType = PIECE_MAP[char.toLowerCase()];
        if (pieceType) {
          board.set(square, {
            type: pieceType,
            color: char === char.toUpperCase() ? 'white' : 'black',
          });
        }
        fileIdx++;
      }
    }
  }

  return board;
}

/**
 * Extract the side to move from a FEN string
 * @param fen - FEN string
 * @returns 'white' or 'black'
 */
export function getSideToMove(fen: string): 'white' | 'black' {
  const parts = fen.split(' ');
  return parts[1] === 'b' ? 'black' : 'white';
}

/**
 * Parse UCI move notation into from/to squares
 * @param uci - UCI move string (e.g., "e2e4")
 * @returns LastMove object with from and to squares
 */
export function parseUciMove(uci: string): LastMove {
  return {
    from: uci.slice(0, 2),
    to: uci.slice(2, 4),
  };
}

/**
 * Find the king's square for a given color
 * @param board - Board map
 * @param color - Color of the king to find
 * @returns Square string or null if not found
 */
export function findKing(
  board: Map<string, PieceInfo>,
  color: 'white' | 'black'
): string | null {
  for (const [square, piece] of board) {
    if (piece.type === 'k' && piece.color === color) {
      return square;
    }
  }
  return null;
}

/** Starting position FEN */
export const STARTING_FEN = 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1';
