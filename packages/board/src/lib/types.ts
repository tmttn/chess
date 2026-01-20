// packages/board/src/lib/types.ts

/** Information about a piece on the board */
export interface PieceInfo {
  type: 'p' | 'n' | 'b' | 'r' | 'q' | 'k';
  color: 'white' | 'black';
}

/** A legal move with destination */
export interface Move {
  from: string;
  to: string;
  uci: string;
  promotion?: string;
}

/** Last move highlight info */
export interface LastMove {
  from: string;
  to: string;
}

/** Board component props */
export interface BoardProps {
  board: Map<string, PieceInfo>;
  legalMoves: Move[];
  flipped?: boolean;
  lastMove?: LastMove | null;
  check?: string | null;
  sideToMove?: 'white' | 'black';
  onMove?: (from: string, to: string, promotion?: string) => void;
}
