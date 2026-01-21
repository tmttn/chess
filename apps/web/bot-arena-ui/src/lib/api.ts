import type { Bot, Match, MatchDetail, Move } from './types';

const BASE_URL = '/api';

/** Single point in Elo rating history */
export interface EloHistoryPoint {
  /** Elo rating at this point in time */
  elo: number;
  /** ISO timestamp of this rating */
  timestamp: string;
}

/** Extended bot profile with Elo history */
export interface BotProfile {
  /** Unique bot identifier/name */
  name: string;
  /** Current ELO rating (starts at 1500) */
  elo_rating: number;
  /** Total games played */
  games_played: number;
  /** Number of wins */
  wins: number;
  /** Number of draws */
  draws: number;
  /** Number of losses */
  losses: number;
  /** Historical Elo ratings over time */
  elo_history: EloHistoryPoint[];
}

/** Record of games between two specific bots */
export interface HeadToHeadRecord {
  /** Bot playing as white */
  white_bot: string;
  /** Bot playing as black */
  black_bot: string;
  /** Number of games won by white */
  white_wins: number;
  /** Number of games won by black */
  black_wins: number;
  /** Number of drawn games */
  draws: number;
  /** Total number of games played */
  games: number;
}

/** Head-to-head matrix data for all bots */
export interface HeadToHeadMatrix {
  /** List of all bot names */
  bots: string[];
  /** Records for each bot pairing */
  records: HeadToHeadRecord[];
}

/** Opening statistics from the database */
export interface OpeningStats {
  /** ECO code (e.g., "B20") */
  eco: string;
  /** Opening name (e.g., "Sicilian Defense") */
  name: string;
  /** Total number of games played with this opening */
  games_played: number;
  /** Number of white wins */
  white_wins: number;
  /** Number of black wins */
  black_wins: number;
  /** Number of draws */
  draws: number;
}

/** Result of Stockfish analysis for a position */
export interface AnalysisResult {
  /** The FEN string of the analyzed position */
  fen: string;
  /** Search depth used for analysis */
  depth: number;
  /** Score in centipawns (null if mate is found) */
  score_cp: number | null;
  /** Moves until mate (null if no mate found, positive = white winning) */
  score_mate: number | null;
  /** Best move in UCI notation */
  best_move: string;
  /** Principal variation (best line) */
  pv: string[];
}

/** Request body for creating a new match */
export interface CreateMatchRequest {
  /** Name of the bot playing as white */
  white_bot: string;
  /** Name of the bot playing as black */
  black_bot: string;
  /** Number of games in the match */
  games: number;
  /** Time per move in milliseconds (optional) */
  movetime_ms?: number;
  /** Opening book ID to use (optional) */
  opening_id?: string;
}

/**
 * Fetch JSON from the API with type safety
 * @param url - API endpoint path (without base URL)
 * @returns Parsed JSON response
 * @throws Error if response is not OK
 */
async function fetchJson<T>(url: string): Promise<T> {
  const response = await fetch(`${BASE_URL}${url}`);
  if (!response.ok) {
    throw new Error(`API error: ${response.status} ${response.statusText}`);
  }
  return response.json();
}

/**
 * Get the URL for exporting a resource as HTML
 * @param type - Type of resource to export ('match', 'game', or 'bot')
 * @param id - Resource identifier (match/game UUID or bot name)
 * @returns Full export URL
 */
export function getExportUrl(type: 'match' | 'game' | 'bot', id: string): string {
  return `${BASE_URL}/export/${type}/${encodeURIComponent(id)}`;
}

/** API client for bot arena server */
export const api = {
  /**
   * Get all registered bots sorted by ELO rating
   * @returns List of bots
   */
  getBots(): Promise<Bot[]> {
    return fetchJson('/bots');
  },

  /**
   * Get a specific bot profile by name including Elo history
   * @param name - Bot name
   * @returns Bot profile with Elo history
   */
  getBot(name: string): Promise<BotProfile> {
    return fetchJson(`/bots/${encodeURIComponent(name)}`);
  },

  /**
   * Get matches with optional filtering
   * @param params - Optional filter parameters
   * @param params.bot - Filter by bot name
   * @param params.limit - Maximum number of results
   * @param params.offset - Pagination offset
   * @returns List of matches
   */
  getMatches(params?: { bot?: string; limit?: number; offset?: number }): Promise<Match[]> {
    const searchParams = new URLSearchParams();
    if (params?.bot) searchParams.set('bot', params.bot);
    if (params?.limit) searchParams.set('limit', params.limit.toString());
    if (params?.offset) searchParams.set('offset', params.offset.toString());

    const query = searchParams.toString();
    return fetchJson(`/matches${query ? `?${query}` : ''}`);
  },

  /**
   * Get match details including all games
   * @param id - Match UUID
   * @returns Match with games
   */
  getMatch(id: string): Promise<MatchDetail> {
    return fetchJson(`/matches/${id}`);
  },

  /**
   * Get all moves for a specific game
   * @param gameId - Game UUID
   * @returns List of moves in order
   */
  getGameMoves(gameId: string): Promise<Move[]> {
    return fetchJson(`/games/${gameId}/moves`);
  },

  /**
   * Create a new match between two bots
   * @param req - Match creation parameters
   * @returns Created match details
   */
  async createMatch(req: CreateMatchRequest): Promise<Match> {
    const response = await fetch(`${BASE_URL}/matches`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(req),
    });
    if (!response.ok) {
      throw new Error(`API error: ${response.status} ${response.statusText}`);
    }
    return response.json();
  },

  /**
   * Analyze a chess position using Stockfish
   * @param fen - FEN string of the position to analyze
   * @param depth - Search depth (default: 20)
   * @returns Analysis result with evaluation and best move
   */
  getAnalysis(fen: string, depth: number = 20): Promise<AnalysisResult> {
    const params = new URLSearchParams({ fen, depth: depth.toString() });
    return fetchJson(`/analysis?${params}`);
  },

  /**
   * Get opening statistics
   * @returns List of opening statistics sorted by games played
   */
  getOpenings(): Promise<OpeningStats[]> {
    return fetchJson('/openings');
  },

  /**
   * Get head-to-head statistics between all bots
   * @returns Matrix of head-to-head records
   */
  getHeadToHead(): Promise<HeadToHeadMatrix> {
    return fetchJson('/stats/head-to-head');
  },
};
