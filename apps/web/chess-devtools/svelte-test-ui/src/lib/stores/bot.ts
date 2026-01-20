import { writable, derived, get } from 'svelte/store';
import { gameStore } from './game';

export interface SearchInfo {
  depth: number;
  score: number;
  nodes: number;
  time: number;
  pv: string[];
}

interface BotSession {
  name: string;
  ready: boolean;
}

export interface BotState {
  connected: boolean;
  connecting: boolean;
  availableBots: string[];
  activeSessions: Map<string, BotSession>; // bot name -> session
  whitePlayer: 'human' | string; // 'human' or bot name
  blackPlayer: 'human' | string;
  autoPlay: boolean;
  lastOutput: string[];
  searchInfo: SearchInfo | null;
  error: string | null;
}

const initialState: BotState = {
  connected: false,
  connecting: false,
  availableBots: [],
  activeSessions: new Map(),
  whitePlayer: 'human',
  blackPlayer: 'human',
  autoPlay: false,
  lastOutput: [],
  searchInfo: null,
  error: null
};

function parseInfoLine(line: string): SearchInfo | null {
  if (!line.startsWith('info ')) return null;

  const parts = line.split(' ');
  const info: SearchInfo = { depth: 0, score: 0, nodes: 0, time: 0, pv: [] };

  for (let i = 1; i < parts.length; i++) {
    switch (parts[i]) {
      case 'depth':
        info.depth = parseInt(parts[++i] || '0', 10);
        break;
      case 'score':
        if (parts[i + 1] === 'cp') {
          i++;
          info.score = parseInt(parts[++i] || '0', 10);
        } else if (parts[i + 1] === 'mate') {
          i++;
          const mateIn = parseInt(parts[++i] || '0', 10);
          info.score = mateIn > 0 ? 100000 - mateIn : -100000 - mateIn;
        }
        break;
      case 'nodes':
        info.nodes = parseInt(parts[++i] || '0', 10);
        break;
      case 'time':
        info.time = parseInt(parts[++i] || '0', 10);
        break;
      case 'pv':
        info.pv = parts.slice(i + 1);
        i = parts.length; // Stop parsing
        break;
    }
  }

  return info.depth > 0 ? info : null;
}

function createBotStore() {
  const { subscribe, set, update } = writable<BotState>(initialState);
  let ws: WebSocket | null = null;
  let pendingBotMove = false;
  let currentBotTurn: string | null = null; // Which bot is currently thinking

  function handleMessage(data: any) {
    update(state => {
      const newOutput = [...state.lastOutput.slice(-99), JSON.stringify(data)];

      switch (data.type) {
        case 'bots':
          return { ...state, availableBots: data.bots, connecting: false, connected: true, lastOutput: newOutput };

        case 'connected': {
          const sessions = new Map(state.activeSessions);
          sessions.set(data.bot, { name: data.bot, ready: false });
          return {
            ...state,
            activeSessions: sessions,
            lastOutput: newOutput
          };
        }

        case 'disconnected':
          return {
            ...state,
            lastOutput: newOutput
          };

        case 'error':
          return {
            ...state,
            error: data.message,
            connecting: false,
            lastOutput: newOutput
          };

        case 'uci': {
          const line = data.line;

          // Check for readyok - mark bot as ready
          if (line === 'readyok' && currentBotTurn) {
            const sessions = new Map(state.activeSessions);
            const session = sessions.get(currentBotTurn);
            if (session) {
              sessions.set(currentBotTurn, { ...session, ready: true });
            }
            return { ...state, activeSessions: sessions, lastOutput: newOutput };
          }

          // Parse search info
          const parsedInfo = parseInfoLine(line);
          if (parsedInfo) {
            return { ...state, lastOutput: newOutput, searchInfo: parsedInfo };
          }

          // Check for bestmove
          if (line.startsWith('bestmove ')) {
            const parts = line.split(' ');
            const move = parts[1];
            if (move && move !== '(none)' && move !== '0000' && pendingBotMove) {
              pendingBotMove = false;
              currentBotTurn = null;
              // Make the move in the game
              gameStore.makeMove(move);
            }
            // Clear search info on bestmove
            return { ...state, lastOutput: newOutput, searchInfo: null };
          }

          return { ...state, lastOutput: newOutput };
        }

        default:
          return { ...state, lastOutput: newOutput };
      }
    });
  }

  function sendToBot(msg: object) {
    if (ws && ws.readyState === WebSocket.OPEN) {
      ws.send(JSON.stringify(msg));
    }
  }

  async function ensureBotConnected(botName: string): Promise<boolean> {
    const state = get({ subscribe });

    if (state.activeSessions.has(botName)) {
      return true;
    }

    // Connect to this bot
    sendToBot({ type: 'connect', bot: botName });

    // Wait a bit for connection, then send UCI init
    await new Promise(resolve => setTimeout(resolve, 50));
    sendToBot({ type: 'uci', cmd: 'uci', bot: botName });
    sendToBot({ type: 'uci', cmd: 'isready', bot: botName });

    // Wait for ready
    await new Promise(resolve => setTimeout(resolve, 100));
    return true;
  }

  // Subscribe to game state changes to trigger bot moves
  let unsubGame: (() => void) | null = null;

  async function checkAndTriggerBotMove() {
    const botState = get({ subscribe });
    const gameState = get(gameStore);

    if (!botState.connected || !botState.autoPlay || gameState.isGameOver) {
      return;
    }

    // Check if it's a bot's turn
    const currentPlayer = gameState.sideToMove === 'white'
      ? botState.whitePlayer
      : botState.blackPlayer;

    if (currentPlayer !== 'human' && !pendingBotMove) {
      // It's a bot's turn, request a move
      pendingBotMove = true;
      currentBotTurn = currentPlayer;

      // Ensure bot is connected
      await ensureBotConnected(currentPlayer);

      // Build the moves list from history
      const moves = gameState.moveHistory
        .slice(0, gameState.viewIndex + 1)
        .map(m => m.uci);

      // Send position and go commands to specific bot
      sendToBot({ type: 'uci', cmd: `position startpos${moves.length > 0 ? ' moves ' + moves.join(' ') : ''}`, bot: currentPlayer });
      sendToBot({ type: 'uci', cmd: 'go movetime 500', bot: currentPlayer });
    }
  }

  function setupGameSubscription() {
    if (unsubGame) return;

    unsubGame = gameStore.subscribe(() => {
      checkAndTriggerBotMove();
    });
  }

  return {
    subscribe,

    connect(url: string = 'ws://127.0.0.1:9999') {
      if (ws) {
        ws.close();
      }

      update(s => ({ ...s, connecting: true, error: null }));

      ws = new WebSocket(url);

      ws.onopen = () => {
        // Request list of available bots
        sendToBot({ type: 'list' });
        setupGameSubscription();
      };

      ws.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data);
          handleMessage(data);
        } catch (e) {
          console.error('Failed to parse message:', event.data);
        }
      };

      ws.onerror = () => {
        update(s => ({
          ...s,
          error: 'WebSocket connection error',
          connecting: false
        }));
      };

      ws.onclose = () => {
        update(s => ({
          ...s,
          connected: false,
          connecting: false,
          activeSessions: new Map()
        }));
        ws = null;
      };
    },

    disconnect() {
      if (ws) {
        ws.close();
        ws = null;
      }
      set(initialState);
    },

    setWhitePlayer(player: 'human' | string) {
      update(s => ({ ...s, whitePlayer: player }));
    },

    setBlackPlayer(player: 'human' | string) {
      update(s => ({ ...s, blackPlayer: player }));
    },

    toggleAutoPlay() {
      update(s => {
        const newAutoPlay = !s.autoPlay;
        pendingBotMove = false; // Reset pending state
        currentBotTurn = null;
        return { ...s, autoPlay: newAutoPlay };
      });
      // Immediately check if we need to trigger a bot move
      setTimeout(() => checkAndTriggerBotMove(), 0);
    },

    sendCommand(cmd: string) {
      sendToBot({ type: 'uci', cmd });
    },

    clearError() {
      update(s => ({ ...s, error: null }));
    }
  };
}

export const botStore = createBotStore();

// Derived stores
export const isConnected = derived(botStore, $bot => $bot.connected);
export const availableBots = derived(botStore, $bot => $bot.availableBots);
export const searchInfo = derived(botStore, $bot => $bot.searchInfo);
export const lastOutput = derived(botStore, $bot => $bot.lastOutput);
