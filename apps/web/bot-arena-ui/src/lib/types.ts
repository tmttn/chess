// API response types

/** Bot information with ELO rating and game statistics */
export interface Bot {
  /** Unique bot identifier/name */
  name: string;
  /** Current ELO rating (starts at 1500) */
  elo_rating: number;
  /** Total games played */
  games_played: number;
  /** Number of wins */
  wins: number;
  /** Number of losses */
  losses: number;
  /** Number of draws */
  draws: number;
}

/** Match between two bots */
export interface Match {
  /** Unique match ID (UUID) */
  id: string;
  /** Name of the white player bot */
  white_bot: string;
  /** Name of the black player bot */
  black_bot: string;
  /** Total number of games in the match */
  games_total: number;
  /** White's score (1 for win, 0.5 for draw) */
  white_score: number;
  /** Black's score */
  black_score: number;
  /** Opening book ID if used */
  opening_id: string | null;
  /** Time per move in milliseconds */
  movetime_ms: number;
  /** ISO timestamp when match started */
  started_at: string;
  /** ISO timestamp when match finished */
  finished_at: string | null;
  /** Match status: pending, in_progress, completed */
  status: string;
}

/** Individual game within a match */
export interface Game {
  /** Unique game ID (UUID) */
  id: string;
  /** Parent match ID */
  match_id: string;
  /** Game number within the match (1-indexed) */
  game_number: number;
  /** Game result: "1-0", "0-1", "1/2-1/2", or null if ongoing */
  result: string | null;
  /** Name of the opening played */
  opening_name: string | null;
  /** Full PGN of the game */
  pgn: string | null;
}

/** Chess move with evaluation data */
export interface Move {
  /** Ply number (half-move) */
  ply: number;
  /** Move in UCI notation (e.g., "e2e4") */
  uci: string;
  /** Move in SAN notation (e.g., "e4") */
  san: string | null;
  /** FEN position after the move */
  fen_after: string;
  /** Bot's evaluation in centipawns */
  bot_eval: number | null;
  /** Stockfish evaluation in centipawns */
  stockfish_eval: number | null;
}

/** Match with associated games */
export interface MatchDetail extends Match {
  /** List of games in this match */
  games: Game[];
}
