import { writable } from 'svelte/store';
import type { Move } from './types';

export interface LiveMatchState {
  connected: boolean;
  moves: Move[];
  currentGame: number;
  score: { white: number; black: number };
}

export function createLiveMatchStore(matchId: string) {
  const { subscribe, update } = writable<LiveMatchState>({
    connected: false,
    moves: [],
    currentGame: 1,
    score: { white: 0, black: 0 },
  });

  let ws: WebSocket | null = null;

  function connect() {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    ws = new WebSocket(`${protocol}//${window.location.host}/ws`);

    ws.onopen = () => {
      update(s => ({ ...s, connected: true }));
      ws?.send(JSON.stringify({ type: 'subscribe', match_id: matchId }));
    };

    ws.onclose = () => {
      update(s => ({ ...s, connected: false }));
    };

    ws.onerror = (event) => {
      console.error('WebSocket error:', event);
      update(s => ({ ...s, connected: false }));
    };

    ws.onmessage = (event) => {
      let msg;
      try {
        msg = JSON.parse(event.data);
      } catch (e) {
        console.error('Failed to parse WebSocket message:', e);
        return;
      }

      switch (msg.type) {
        case 'move':
          update(s => ({
            ...s,
            moves: [...s.moves, {
              ply: s.moves.length + 1,
              uci: msg.uci,
              san: null,
              fen_after: '',
              bot_eval: msg.centipawns,  // Note: server sends 'centipawns' not 'eval'
              stockfish_eval: null,
            }],
          }));
          break;

        case 'game_end':
          update(s => ({
            ...s,
            moves: [],
            currentGame: msg.game_num + 1,
          }));
          break;

        case 'match_end':
          const [white, black] = msg.score.split('-').map(Number);
          update(s => ({
            ...s,
            score: { white, black },
          }));
          break;
      }
    };
  }

  function disconnect() {
    if (ws) {
      if (ws.readyState === WebSocket.OPEN) {
        ws.send(JSON.stringify({ type: 'unsubscribe', match_id: matchId }));
      }
      ws.close();
      ws = null;
    }
  }

  return {
    subscribe,
    connect,
    disconnect,
  };
}
