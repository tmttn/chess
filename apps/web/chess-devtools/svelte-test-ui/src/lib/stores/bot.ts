// apps/web/chess-devtools/svelte-test-ui/src/lib/stores/bot.ts
//
// Bot store using @tmttn-chess/bot-client with game store integration

import { writable, derived, get } from 'svelte/store';
import {
  createBotClient,
  type BotClient,
  type BotSession,
  type SearchInfo,
  type BotClientCallbacks,
} from '@tmttn-chess/bot-client';
import { gameStore } from './game';

// Re-export SearchInfo type for consumers
export type { SearchInfo };

export interface BotState {
  connected: boolean;
  connecting: boolean;
  availableBots: string[];
  whitePlayer: 'human' | string;
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
  whitePlayer: 'human',
  blackPlayer: 'human',
  autoPlay: false,
  lastOutput: [],
  searchInfo: null,
  error: null,
};

function createBotStore() {
  const { subscribe, set, update } = writable<BotState>(initialState);

  let client: BotClient | null = null;
  let currentSession: BotSession | null = null;
  let pendingBotMove = false;
  let currentBotTurn: string | null = null;
  let lastSearchInfo: SearchInfo | null = null;

  // Game subscription cleanup
  let unsubGame: (() => void) | null = null;

  /**
   * Check if it's a bot's turn and trigger a move if so.
   */
  async function checkAndTriggerBotMove() {
    const botState = get({ subscribe });
    const gameState = get(gameStore.liveFen);
    const isGameOverVal = get(gameStore.isGameOver);
    const side = get(gameStore.sideToMove);
    const history = get(gameStore.moveHistory);

    if (!client || !botState.connected || !botState.autoPlay || isGameOverVal) {
      return;
    }

    const currentPlayer = side === 'white' ? botState.whitePlayer : botState.blackPlayer;

    if (currentPlayer !== 'human' && !pendingBotMove) {
      pendingBotMove = true;
      currentBotTurn = currentPlayer;

      try {
        // Get or create session for this bot
        if (!currentSession || currentSession.name !== currentPlayer) {
          currentSession = await client.startSession(currentPlayer);
        }

        // Build moves list from history
        const moves = history.map((m) => m.uci);

        // Send position and start search
        currentSession.sendPosition(moves);
        currentSession.go({ movetime: 500 });
      } catch (e) {
        console.error('Failed to trigger bot move:', e);
        pendingBotMove = false;
        currentBotTurn = null;
      }
    }
  }

  /**
   * Set up game store subscription to trigger bot moves.
   */
  function setupGameSubscription() {
    if (unsubGame) return;
    unsubGame = gameStore.liveFen.subscribe(() => {
      checkAndTriggerBotMove();
    });
  }

  /**
   * Create callbacks for the bot client.
   */
  function createCallbacks(): BotClientCallbacks {
    return {
      onConnect: () => {
        update((s) => ({ ...s, connecting: false, connected: true }));
        setupGameSubscription();
      },

      onDisconnect: () => {
        update((s) => ({
          ...s,
          connected: false,
          connecting: false,
          availableBots: [],
        }));
        currentSession = null;
      },

      onError: (message) => {
        update((s) => ({
          ...s,
          error: message,
          connecting: false,
        }));
      },

      onBots: (bots) => {
        update((s) => ({
          ...s,
          availableBots: bots,
          lastOutput: [...s.lastOutput.slice(-99), `Bots: ${bots.join(', ')}`],
        }));
      },

      onSearchInfo: (info) => {
        lastSearchInfo = info;
        update((s) => ({
          ...s,
          searchInfo: info,
          lastOutput: [
            ...s.lastOutput.slice(-99),
            `depth=${info.depth} score=${info.score} nodes=${info.nodes}`,
          ],
        }));
      },

      onBestMove: (move, searchInfo) => {
        if (pendingBotMove) {
          pendingBotMove = false;
          currentBotTurn = null;

          // Make the move in the game
          gameStore.makeMove(move);

          // Attach search info to the move
          if (searchInfo) {
            gameStore.attachSearchInfoToLastMove(searchInfo);
          }
        }

        // Clear search info
        update((s) => ({
          ...s,
          searchInfo: null,
          lastOutput: [...s.lastOutput.slice(-99), `bestmove ${move}`],
        }));
        lastSearchInfo = null;
      },
    };
  }

  return {
    subscribe,

    connect(url: string = 'ws://127.0.0.1:9999') {
      if (client) {
        client.disconnect();
      }

      update((s) => ({ ...s, connecting: true, error: null }));

      client = createBotClient({
        url,
        ...createCallbacks(),
      });

      client.connect().catch((e) => {
        update((s) => ({
          ...s,
          error: e.message || 'Connection failed',
          connecting: false,
        }));
      });
    },

    disconnect() {
      if (client) {
        client.disconnect();
        client = null;
      }
      if (unsubGame) {
        unsubGame();
        unsubGame = null;
      }
      currentSession = null;
      pendingBotMove = false;
      currentBotTurn = null;
      lastSearchInfo = null;
      set(initialState);
    },

    setWhitePlayer(player: 'human' | string) {
      update((s) => ({ ...s, whitePlayer: player }));
      pendingBotMove = false;
      currentBotTurn = null;
      setTimeout(() => checkAndTriggerBotMove(), 0);
    },

    setBlackPlayer(player: 'human' | string) {
      update((s) => ({ ...s, blackPlayer: player }));
      pendingBotMove = false;
      currentBotTurn = null;
      setTimeout(() => checkAndTriggerBotMove(), 0);
    },

    toggleAutoPlay() {
      update((s) => {
        pendingBotMove = false;
        currentBotTurn = null;
        return { ...s, autoPlay: !s.autoPlay };
      });
      setTimeout(() => checkAndTriggerBotMove(), 0);
    },

    sendCommand(cmd: string) {
      if (client) {
        client.sendRawCommand(cmd);
      }
    },

    clearError() {
      update((s) => ({ ...s, error: null }));
    },
  };
}

export const botStore = createBotStore();

// Derived stores
export const isConnected = derived(botStore, ($bot) => $bot.connected);
export const availableBots = derived(botStore, ($bot) => $bot.availableBots);
export const searchInfo = derived(botStore, ($bot) => $bot.searchInfo);
export const lastOutput = derived(botStore, ($bot) => $bot.lastOutput);
