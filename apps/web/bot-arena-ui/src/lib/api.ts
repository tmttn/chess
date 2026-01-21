import type { Bot, Match, MatchDetail, Move } from './types';

const BASE_URL = '/api';

async function fetchJson<T>(url: string): Promise<T> {
  const response = await fetch(`${BASE_URL}${url}`);
  if (!response.ok) {
    throw new Error(`API error: ${response.status}`);
  }
  return response.json();
}

export const api = {
  // Bots
  getBots(): Promise<Bot[]> {
    return fetchJson('/bots');
  },

  getBot(name: string): Promise<Bot> {
    return fetchJson(`/bots/${encodeURIComponent(name)}`);
  },

  // Matches
  getMatches(params?: { bot?: string; limit?: number; offset?: number }): Promise<Match[]> {
    const searchParams = new URLSearchParams();
    if (params?.bot) searchParams.set('bot', params.bot);
    if (params?.limit) searchParams.set('limit', params.limit.toString());
    if (params?.offset) searchParams.set('offset', params.offset.toString());

    const query = searchParams.toString();
    return fetchJson(`/matches${query ? `?${query}` : ''}`);
  },

  getMatch(id: string): Promise<MatchDetail> {
    return fetchJson(`/matches/${id}`);
  },

  getGameMoves(gameId: string): Promise<Move[]> {
    return fetchJson(`/games/${gameId}/moves`);
  },
};
