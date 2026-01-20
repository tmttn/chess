// packages/bot-client/src/client.ts

import { writable, get } from 'svelte/store';
import type {
  BotClient,
  BotClientConfig,
  BotSession,
  GoOptions,
  SearchInfo
} from './types';

/** Parse UCI info line to extract search information */
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
          // Convert mate score: positive = mating, negative = being mated
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
        i = parts.length; // Stop parsing after pv
        break;
    }
  }

  // Only return if we got meaningful depth info
  return info.depth > 0 ? info : null;
}

/** Internal state for a bot session */
interface SessionState {
  name: string;
  ready: boolean;
}

/**
 * Create a new bot client instance.
 *
 * The client manages WebSocket connections to the bot-bridge server
 * and provides an event-driven API for interacting with UCI engines.
 */
export function createBotClient(config: BotClientConfig = {}): BotClient {
  const url = config.url ?? 'ws://127.0.0.1:9999';

  // Internal state
  const connectedStore = writable(false);
  const connectingStore = writable(false);
  const availableBotsStore = writable<string[]>([]);
  const searchInfoStore = writable<SearchInfo | null>(null);
  const errorStore = writable<string | null>(null);
  const sessionsStore = writable<Map<string, SessionState>>(new Map());

  let ws: WebSocket | null = null;
  let lastSearchInfo: SearchInfo | null = null;

  // Callbacks
  const {
    onConnect,
    onDisconnect,
    onError,
    onBots,
    onSearchInfo,
    onBestMove
  } = config;

  /** Send a message to the WebSocket server */
  function send(msg: object): void {
    if (ws && ws.readyState === WebSocket.OPEN) {
      ws.send(JSON.stringify(msg));
    }
  }

  /** Handle incoming messages from the server */
  function handleMessage(data: Record<string, unknown>): void {
    switch (data.type) {
      case 'bots': {
        const bots = data.bots as string[];
        availableBotsStore.set(bots);
        connectingStore.set(false);
        connectedStore.set(true);
        onBots?.(bots);
        break;
      }

      case 'connected': {
        const botName = data.bot as string;
        sessionsStore.update((sessions) => {
          const newSessions = new Map(sessions);
          newSessions.set(botName, { name: botName, ready: false });
          return newSessions;
        });
        break;
      }

      case 'disconnected': {
        const botName = data.bot as string;
        sessionsStore.update((sessions) => {
          const newSessions = new Map(sessions);
          newSessions.delete(botName);
          return newSessions;
        });
        break;
      }

      case 'error': {
        const message = data.message as string;
        errorStore.set(message);
        connectingStore.set(false);
        onError?.(message);
        break;
      }

      case 'uci': {
        const line = data.line as string;
        const botName = data.bot as string | undefined;

        // Check for readyok - mark bot as ready
        if (line === 'readyok' && botName) {
          sessionsStore.update((sessions) => {
            const session = sessions.get(botName);
            if (session) {
              const newSessions = new Map(sessions);
              newSessions.set(botName, { ...session, ready: true });
              return newSessions;
            }
            return sessions;
          });
          break;
        }

        // Parse search info
        const parsedInfo = parseInfoLine(line);
        if (parsedInfo) {
          lastSearchInfo = parsedInfo;
          searchInfoStore.set(parsedInfo);
          onSearchInfo?.(parsedInfo);
          break;
        }

        // Check for bestmove
        if (line.startsWith('bestmove ')) {
          const parts = line.split(' ');
          const move = parts[1];

          if (move && move !== '(none)' && move !== '0000') {
            onBestMove?.(move, lastSearchInfo);
          }

          // Clear search info after bestmove
          searchInfoStore.set(null);
          lastSearchInfo = null;
        }
        break;
      }
    }
  }

  /** Connect to the bot-bridge server */
  function connect(): Promise<void> {
    return new Promise((resolve, reject) => {
      if (ws) {
        ws.close();
      }

      connectingStore.set(true);
      errorStore.set(null);

      ws = new WebSocket(url);

      ws.onopen = () => {
        // Request list of available bots
        send({ type: 'list' });
        onConnect?.();
        resolve();
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
        const message = 'WebSocket connection error';
        errorStore.set(message);
        connectingStore.set(false);
        onError?.(message);
        reject(new Error(message));
      };

      ws.onclose = () => {
        connectedStore.set(false);
        connectingStore.set(false);
        sessionsStore.set(new Map());
        ws = null;
        onDisconnect?.();
      };
    });
  }

  /** Disconnect from the bot-bridge server */
  function disconnect(): void {
    if (ws) {
      ws.close();
      ws = null;
    }
    connectedStore.set(false);
    connectingStore.set(false);
    availableBotsStore.set([]);
    sessionsStore.set(new Map());
    searchInfoStore.set(null);
    errorStore.set(null);
    lastSearchInfo = null;
  }

  /** Send a raw UCI command */
  function sendRawCommand(cmd: string, bot?: string): void {
    send({ type: 'uci', cmd, bot });
  }

  /**
   * Start a session with a specific bot.
   * This connects to the bot and waits for it to be ready.
   */
  async function startSession(botName: string): Promise<BotSession> {
    const sessions = get(sessionsStore);

    // Check if already connected
    if (!sessions.has(botName)) {
      // Connect to this bot
      send({ type: 'connect', bot: botName });

      // Wait a bit for connection confirmation
      await new Promise((resolve) => setTimeout(resolve, 50));

      // Send UCI initialization
      send({ type: 'uci', cmd: 'uci', bot: botName });
      send({ type: 'uci', cmd: 'isready', bot: botName });

      // Wait for ready
      await new Promise((resolve) => setTimeout(resolve, 100));
    }

    // Create the session object
    const session: BotSession = {
      name: botName,

      sendPosition(moves: string[]): void {
        const posCmd =
          moves.length > 0
            ? `position startpos moves ${moves.join(' ')}`
            : 'position startpos';
        send({ type: 'uci', cmd: posCmd, bot: botName });
      },

      go(options: GoOptions): void {
        let goCmd = 'go';

        if (options.infinite) {
          goCmd += ' infinite';
        } else if (options.movetime !== undefined) {
          goCmd += ` movetime ${options.movetime}`;
        } else if (options.depth !== undefined) {
          goCmd += ` depth ${options.depth}`;
        }

        send({ type: 'uci', cmd: goCmd, bot: botName });
      },

      stop(): void {
        send({ type: 'uci', cmd: 'stop', bot: botName });
      },

      close(): void {
        send({ type: 'disconnect', bot: botName });
        sessionsStore.update((sessions) => {
          const newSessions = new Map(sessions);
          newSessions.delete(botName);
          return newSessions;
        });
      }
    };

    return session;
  }

  return {
    // State stores (read-only)
    connected: { subscribe: connectedStore.subscribe },
    connecting: { subscribe: connectingStore.subscribe },
    availableBots: { subscribe: availableBotsStore.subscribe },
    searchInfo: { subscribe: searchInfoStore.subscribe },
    error: { subscribe: errorStore.subscribe },

    // Methods
    connect,
    disconnect,
    startSession,
    sendRawCommand
  };
}
