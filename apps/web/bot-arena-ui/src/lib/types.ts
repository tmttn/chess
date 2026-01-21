// API response types

export interface Bot {
  name: string;
  elo_rating: number;
  games_played: number;
  wins: number;
  losses: number;
  draws: number;
}

export interface Match {
  id: string;
  white_bot: string;
  black_bot: string;
  games_total: number;
  white_score: number;
  black_score: number;
  opening_id: string | null;
  movetime_ms: number;
  started_at: string;
  finished_at: string | null;
  status: string;
}

export interface Game {
  id: string;
  match_id: string;
  game_number: number;
  result: string | null;
  opening_name: string | null;
  pgn: string | null;
}

export interface Move {
  ply: number;
  uci: string;
  san: string | null;
  fen_after: string;
  bot_eval: number | null;
  stockfish_eval: number | null;
}

export interface MatchDetail extends Match {
  games: Game[];
}
