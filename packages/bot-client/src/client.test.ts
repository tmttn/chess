// packages/bot-client/src/client.test.ts

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { get } from 'svelte/store';
import { createBotClient } from './client';

/** Mock WebSocket instance */
interface MockWebSocketInstance {
  url: string;
  readyState: number;
  onopen: (() => void) | null;
  onmessage: ((event: { data: string }) => void) | null;
  onerror: (() => void) | null;
  onclose: (() => void) | null;
  send: ReturnType<typeof vi.fn>;
  close: ReturnType<typeof vi.fn>;
}

/** Helper class to capture and control WebSocket instances */
class MockWebSocket implements MockWebSocketInstance {
  static instances: MockWebSocket[] = [];
  static CONNECTING = 0;
  static OPEN = 1;
  static CLOSING = 2;
  static CLOSED = 3;

  url: string;
  readyState = MockWebSocket.CONNECTING;
  onopen: (() => void) | null = null;
  onmessage: ((event: { data: string }) => void) | null = null;
  onerror: (() => void) | null = null;
  onclose: (() => void) | null = null;
  send = vi.fn();
  close = vi.fn();

  constructor(url: string) {
    this.url = url;
    MockWebSocket.instances.push(this);
  }

  /** Simulate the connection opening */
  simulateOpen(): void {
    this.readyState = MockWebSocket.OPEN;
    this.onopen?.();
  }

  /** Simulate receiving a message */
  simulateMessage(data: unknown): void {
    this.onmessage?.({ data: JSON.stringify(data) });
  }

  /** Simulate an error */
  simulateError(): void {
    this.onerror?.();
  }

  /** Simulate the connection closing */
  simulateClose(): void {
    this.readyState = MockWebSocket.CLOSED;
    this.onclose?.();
  }

  static reset(): void {
    MockWebSocket.instances = [];
  }

  static get lastInstance(): MockWebSocket | undefined {
    return MockWebSocket.instances[MockWebSocket.instances.length - 1];
  }
}

describe('createBotClient', () => {
  let originalWebSocket: typeof WebSocket;

  beforeEach(() => {
    MockWebSocket.reset();
    originalWebSocket = globalThis.WebSocket;
    globalThis.WebSocket = MockWebSocket as unknown as typeof WebSocket;
  });

  afterEach(() => {
    globalThis.WebSocket = originalWebSocket;
  });

  describe('initial state', () => {
    it('starts in disconnected state', () => {
      const client = createBotClient();

      expect(get(client.connected)).toBe(false);
      expect(get(client.connecting)).toBe(false);
      expect(get(client.availableBots)).toEqual([]);
      expect(get(client.searchInfo)).toBeNull();
      expect(get(client.error)).toBeNull();
    });

    it('uses default URL when not provided', () => {
      const client = createBotClient();
      client.connect();

      expect(MockWebSocket.lastInstance?.url).toBe('ws://127.0.0.1:9999');
    });

    it('uses custom URL when provided', () => {
      const client = createBotClient({ url: 'ws://custom:8080' });
      client.connect();

      expect(MockWebSocket.lastInstance?.url).toBe('ws://custom:8080');
    });
  });

  describe('connection flow', () => {
    it('sets connecting state during connection', () => {
      const client = createBotClient();
      client.connect();

      expect(get(client.connecting)).toBe(true);
      expect(get(client.connected)).toBe(false);
    });

    it('resolves connect promise when connection opens', async () => {
      const client = createBotClient();
      const connectPromise = client.connect();

      MockWebSocket.lastInstance?.simulateOpen();

      await expect(connectPromise).resolves.toBeUndefined();
    });

    it('calls onConnect callback when connection opens', async () => {
      const onConnect = vi.fn();
      const client = createBotClient({ onConnect });

      const connectPromise = client.connect();
      MockWebSocket.lastInstance?.simulateOpen();
      await connectPromise;

      expect(onConnect).toHaveBeenCalledTimes(1);
    });

    it('sends list request on connection open', async () => {
      const client = createBotClient();
      const connectPromise = client.connect();

      MockWebSocket.lastInstance?.simulateOpen();
      await connectPromise;

      expect(MockWebSocket.lastInstance?.send).toHaveBeenCalledWith(
        JSON.stringify({ type: 'list' })
      );
    });

    it('sets connected state after receiving bots list', async () => {
      const client = createBotClient();
      const connectPromise = client.connect();

      MockWebSocket.lastInstance?.simulateOpen();
      await connectPromise;

      MockWebSocket.lastInstance?.simulateMessage({
        type: 'bots',
        bots: ['stockfish', 'lc0']
      });

      expect(get(client.connected)).toBe(true);
      expect(get(client.connecting)).toBe(false);
      expect(get(client.availableBots)).toEqual(['stockfish', 'lc0']);
    });

    it('calls onBots callback with bot list', async () => {
      const onBots = vi.fn();
      const client = createBotClient({ onBots });

      const connectPromise = client.connect();
      MockWebSocket.lastInstance?.simulateOpen();
      await connectPromise;

      MockWebSocket.lastInstance?.simulateMessage({
        type: 'bots',
        bots: ['stockfish']
      });

      expect(onBots).toHaveBeenCalledWith(['stockfish']);
    });
  });

  describe('error handling', () => {
    it('rejects connect promise on WebSocket error', async () => {
      const client = createBotClient();
      const connectPromise = client.connect();

      MockWebSocket.lastInstance?.simulateError();

      await expect(connectPromise).rejects.toThrow('WebSocket connection error');
    });

    it('sets error state on WebSocket error', async () => {
      const client = createBotClient();
      const connectPromise = client.connect();

      MockWebSocket.lastInstance?.simulateError();

      try {
        await connectPromise;
      } catch {
        // Expected
      }

      expect(get(client.error)).toBe('WebSocket connection error');
      expect(get(client.connecting)).toBe(false);
    });

    it('calls onError callback on WebSocket error', async () => {
      const onError = vi.fn();
      const client = createBotClient({ onError });

      const connectPromise = client.connect();
      MockWebSocket.lastInstance?.simulateError();

      try {
        await connectPromise;
      } catch {
        // Expected
      }

      expect(onError).toHaveBeenCalledWith('WebSocket connection error');
    });

    it('handles server error messages', async () => {
      const onError = vi.fn();
      const client = createBotClient({ onError });

      const connectPromise = client.connect();
      MockWebSocket.lastInstance?.simulateOpen();
      await connectPromise;

      MockWebSocket.lastInstance?.simulateMessage({
        type: 'error',
        message: 'Bot not found'
      });

      expect(get(client.error)).toBe('Bot not found');
      expect(onError).toHaveBeenCalledWith('Bot not found');
    });
  });

  describe('disconnect', () => {
    it('cleans up state on disconnect', async () => {
      const client = createBotClient();
      const connectPromise = client.connect();

      MockWebSocket.lastInstance?.simulateOpen();
      await connectPromise;

      MockWebSocket.lastInstance?.simulateMessage({
        type: 'bots',
        bots: ['stockfish']
      });

      expect(get(client.connected)).toBe(true);

      client.disconnect();

      expect(get(client.connected)).toBe(false);
      expect(get(client.connecting)).toBe(false);
      expect(get(client.availableBots)).toEqual([]);
      expect(get(client.searchInfo)).toBeNull();
      expect(get(client.error)).toBeNull();
    });

    it('calls onDisconnect callback when connection closes', async () => {
      const onDisconnect = vi.fn();
      const client = createBotClient({ onDisconnect });

      const connectPromise = client.connect();
      MockWebSocket.lastInstance?.simulateOpen();
      await connectPromise;

      MockWebSocket.lastInstance?.simulateClose();

      expect(onDisconnect).toHaveBeenCalledTimes(1);
    });

    it('resets state when connection closes unexpectedly', async () => {
      const client = createBotClient();
      const connectPromise = client.connect();

      MockWebSocket.lastInstance?.simulateOpen();
      await connectPromise;

      MockWebSocket.lastInstance?.simulateMessage({
        type: 'bots',
        bots: ['stockfish']
      });

      expect(get(client.connected)).toBe(true);

      MockWebSocket.lastInstance?.simulateClose();

      expect(get(client.connected)).toBe(false);
      expect(get(client.connecting)).toBe(false);
    });
  });

  describe('search info parsing', () => {
    it('parses UCI info line with depth and score', async () => {
      const onSearchInfo = vi.fn();
      const client = createBotClient({ onSearchInfo });

      const connectPromise = client.connect();
      MockWebSocket.lastInstance?.simulateOpen();
      await connectPromise;

      MockWebSocket.lastInstance?.simulateMessage({
        type: 'uci',
        line: 'info depth 10 score cp 50 nodes 12345 time 100 pv e2e4 e7e5',
        bot: 'stockfish'
      });

      expect(onSearchInfo).toHaveBeenCalledWith({
        depth: 10,
        score: 50,
        nodes: 12345,
        time: 100,
        pv: ['e2e4', 'e7e5']
      });

      expect(get(client.searchInfo)).toEqual({
        depth: 10,
        score: 50,
        nodes: 12345,
        time: 100,
        pv: ['e2e4', 'e7e5']
      });
    });

    it('parses mate score as large positive value', async () => {
      const onSearchInfo = vi.fn();
      const client = createBotClient({ onSearchInfo });

      const connectPromise = client.connect();
      MockWebSocket.lastInstance?.simulateOpen();
      await connectPromise;

      MockWebSocket.lastInstance?.simulateMessage({
        type: 'uci',
        line: 'info depth 20 score mate 3 nodes 5000 time 50 pv d1h5 f7f6 h5f7',
        bot: 'stockfish'
      });

      expect(onSearchInfo).toHaveBeenCalledWith({
        depth: 20,
        score: 100000 - 3, // mate in 3
        nodes: 5000,
        time: 50,
        pv: ['d1h5', 'f7f6', 'h5f7']
      });
    });

    it('parses negative mate score as large negative value', async () => {
      const onSearchInfo = vi.fn();
      const client = createBotClient({ onSearchInfo });

      const connectPromise = client.connect();
      MockWebSocket.lastInstance?.simulateOpen();
      await connectPromise;

      MockWebSocket.lastInstance?.simulateMessage({
        type: 'uci',
        line: 'info depth 15 score mate -2 nodes 3000 time 30 pv e1e2 d8d1',
        bot: 'stockfish'
      });

      expect(onSearchInfo).toHaveBeenCalledWith({
        depth: 15,
        score: -100000 - (-2), // being mated in 2
        nodes: 3000,
        time: 30,
        pv: ['e1e2', 'd8d1']
      });
    });

    it('ignores info lines without depth', async () => {
      const onSearchInfo = vi.fn();
      const client = createBotClient({ onSearchInfo });

      const connectPromise = client.connect();
      MockWebSocket.lastInstance?.simulateOpen();
      await connectPromise;

      MockWebSocket.lastInstance?.simulateMessage({
        type: 'uci',
        line: 'info string Stockfish ready',
        bot: 'stockfish'
      });

      expect(onSearchInfo).not.toHaveBeenCalled();
      expect(get(client.searchInfo)).toBeNull();
    });

    it('ignores non-info lines', async () => {
      const onSearchInfo = vi.fn();
      const client = createBotClient({ onSearchInfo });

      const connectPromise = client.connect();
      MockWebSocket.lastInstance?.simulateOpen();
      await connectPromise;

      MockWebSocket.lastInstance?.simulateMessage({
        type: 'uci',
        line: 'uciok',
        bot: 'stockfish'
      });

      expect(onSearchInfo).not.toHaveBeenCalled();
    });
  });

  describe('bestmove handling', () => {
    it('calls onBestMove with move and last search info', async () => {
      const onBestMove = vi.fn();
      const client = createBotClient({ onBestMove });

      const connectPromise = client.connect();
      MockWebSocket.lastInstance?.simulateOpen();
      await connectPromise;

      // Send search info first
      MockWebSocket.lastInstance?.simulateMessage({
        type: 'uci',
        line: 'info depth 10 score cp 25 nodes 10000 time 100 pv e2e4',
        bot: 'stockfish'
      });

      // Then bestmove
      MockWebSocket.lastInstance?.simulateMessage({
        type: 'uci',
        line: 'bestmove e2e4 ponder e7e5',
        bot: 'stockfish'
      });

      expect(onBestMove).toHaveBeenCalledWith('e2e4', {
        depth: 10,
        score: 25,
        nodes: 10000,
        time: 100,
        pv: ['e2e4']
      });
    });

    it('calls onBestMove with null searchInfo when no prior info', async () => {
      const onBestMove = vi.fn();
      const client = createBotClient({ onBestMove });

      const connectPromise = client.connect();
      MockWebSocket.lastInstance?.simulateOpen();
      await connectPromise;

      MockWebSocket.lastInstance?.simulateMessage({
        type: 'uci',
        line: 'bestmove d2d4',
        bot: 'stockfish'
      });

      expect(onBestMove).toHaveBeenCalledWith('d2d4', null);
    });

    it('clears search info after bestmove', async () => {
      const client = createBotClient();

      const connectPromise = client.connect();
      MockWebSocket.lastInstance?.simulateOpen();
      await connectPromise;

      MockWebSocket.lastInstance?.simulateMessage({
        type: 'uci',
        line: 'info depth 10 score cp 25 nodes 10000 time 100 pv e2e4',
        bot: 'stockfish'
      });

      expect(get(client.searchInfo)).not.toBeNull();

      MockWebSocket.lastInstance?.simulateMessage({
        type: 'uci',
        line: 'bestmove e2e4',
        bot: 'stockfish'
      });

      expect(get(client.searchInfo)).toBeNull();
    });

    it('ignores bestmove (none)', async () => {
      const onBestMove = vi.fn();
      const client = createBotClient({ onBestMove });

      const connectPromise = client.connect();
      MockWebSocket.lastInstance?.simulateOpen();
      await connectPromise;

      MockWebSocket.lastInstance?.simulateMessage({
        type: 'uci',
        line: 'bestmove (none)',
        bot: 'stockfish'
      });

      expect(onBestMove).not.toHaveBeenCalled();
    });

    it('ignores bestmove 0000', async () => {
      const onBestMove = vi.fn();
      const client = createBotClient({ onBestMove });

      const connectPromise = client.connect();
      MockWebSocket.lastInstance?.simulateOpen();
      await connectPromise;

      MockWebSocket.lastInstance?.simulateMessage({
        type: 'uci',
        line: 'bestmove 0000',
        bot: 'stockfish'
      });

      expect(onBestMove).not.toHaveBeenCalled();
    });
  });

  describe('session management', () => {
    it('handles bot connected message', async () => {
      const client = createBotClient();

      const connectPromise = client.connect();
      MockWebSocket.lastInstance?.simulateOpen();
      await connectPromise;

      MockWebSocket.lastInstance?.simulateMessage({
        type: 'connected',
        bot: 'stockfish'
      });

      // Session should be tracked internally (not directly exposed)
      // This is just testing the message handling doesn't throw
    });

    it('handles bot disconnected message', async () => {
      const client = createBotClient();

      const connectPromise = client.connect();
      MockWebSocket.lastInstance?.simulateOpen();
      await connectPromise;

      MockWebSocket.lastInstance?.simulateMessage({
        type: 'connected',
        bot: 'stockfish'
      });

      MockWebSocket.lastInstance?.simulateMessage({
        type: 'disconnected',
        bot: 'stockfish'
      });

      // Session should be removed internally
    });

    it('handles readyok message', async () => {
      const client = createBotClient();

      const connectPromise = client.connect();
      MockWebSocket.lastInstance?.simulateOpen();
      await connectPromise;

      MockWebSocket.lastInstance?.simulateMessage({
        type: 'connected',
        bot: 'stockfish'
      });

      MockWebSocket.lastInstance?.simulateMessage({
        type: 'uci',
        line: 'readyok',
        bot: 'stockfish'
      });

      // Bot should be marked as ready internally
    });
  });

  describe('startSession', () => {
    it('sends connect command for new bot', async () => {
      const client = createBotClient();

      const connectPromise = client.connect();
      MockWebSocket.lastInstance?.simulateOpen();
      await connectPromise;

      MockWebSocket.lastInstance?.simulateMessage({
        type: 'bots',
        bots: ['stockfish']
      });

      // Clear the send calls from connect
      MockWebSocket.lastInstance?.send.mockClear();

      await client.startSession('stockfish');

      expect(MockWebSocket.lastInstance?.send).toHaveBeenCalledWith(
        JSON.stringify({ type: 'connect', bot: 'stockfish' })
      );
    });

    it('returns session with bot name', async () => {
      const client = createBotClient();

      const connectPromise = client.connect();
      MockWebSocket.lastInstance?.simulateOpen();
      await connectPromise;

      const session = await client.startSession('stockfish');

      expect(session.name).toBe('stockfish');
    });

    it('session.sendPosition sends position command', async () => {
      const client = createBotClient();

      const connectPromise = client.connect();
      MockWebSocket.lastInstance?.simulateOpen();
      await connectPromise;

      const session = await client.startSession('stockfish');

      MockWebSocket.lastInstance?.send.mockClear();

      session.sendPosition(['e2e4', 'e7e5']);

      expect(MockWebSocket.lastInstance?.send).toHaveBeenCalledWith(
        JSON.stringify({
          type: 'uci',
          cmd: 'position startpos moves e2e4 e7e5',
          bot: 'stockfish'
        })
      );
    });

    it('session.sendPosition with empty moves sends startpos only', async () => {
      const client = createBotClient();

      const connectPromise = client.connect();
      MockWebSocket.lastInstance?.simulateOpen();
      await connectPromise;

      const session = await client.startSession('stockfish');

      MockWebSocket.lastInstance?.send.mockClear();

      session.sendPosition([]);

      expect(MockWebSocket.lastInstance?.send).toHaveBeenCalledWith(
        JSON.stringify({
          type: 'uci',
          cmd: 'position startpos',
          bot: 'stockfish'
        })
      );
    });

    it('session.go sends go command with movetime', async () => {
      const client = createBotClient();

      const connectPromise = client.connect();
      MockWebSocket.lastInstance?.simulateOpen();
      await connectPromise;

      const session = await client.startSession('stockfish');

      MockWebSocket.lastInstance?.send.mockClear();

      session.go({ movetime: 1000 });

      expect(MockWebSocket.lastInstance?.send).toHaveBeenCalledWith(
        JSON.stringify({
          type: 'uci',
          cmd: 'go movetime 1000',
          bot: 'stockfish'
        })
      );
    });

    it('session.go sends go command with depth', async () => {
      const client = createBotClient();

      const connectPromise = client.connect();
      MockWebSocket.lastInstance?.simulateOpen();
      await connectPromise;

      const session = await client.startSession('stockfish');

      MockWebSocket.lastInstance?.send.mockClear();

      session.go({ depth: 20 });

      expect(MockWebSocket.lastInstance?.send).toHaveBeenCalledWith(
        JSON.stringify({
          type: 'uci',
          cmd: 'go depth 20',
          bot: 'stockfish'
        })
      );
    });

    it('session.go sends go infinite', async () => {
      const client = createBotClient();

      const connectPromise = client.connect();
      MockWebSocket.lastInstance?.simulateOpen();
      await connectPromise;

      const session = await client.startSession('stockfish');

      MockWebSocket.lastInstance?.send.mockClear();

      session.go({ infinite: true });

      expect(MockWebSocket.lastInstance?.send).toHaveBeenCalledWith(
        JSON.stringify({
          type: 'uci',
          cmd: 'go infinite',
          bot: 'stockfish'
        })
      );
    });

    it('session.stop sends stop command', async () => {
      const client = createBotClient();

      const connectPromise = client.connect();
      MockWebSocket.lastInstance?.simulateOpen();
      await connectPromise;

      const session = await client.startSession('stockfish');

      MockWebSocket.lastInstance?.send.mockClear();

      session.stop();

      expect(MockWebSocket.lastInstance?.send).toHaveBeenCalledWith(
        JSON.stringify({
          type: 'uci',
          cmd: 'stop',
          bot: 'stockfish'
        })
      );
    });

    it('session.close sends disconnect command', async () => {
      const client = createBotClient();

      const connectPromise = client.connect();
      MockWebSocket.lastInstance?.simulateOpen();
      await connectPromise;

      const session = await client.startSession('stockfish');

      MockWebSocket.lastInstance?.send.mockClear();

      session.close();

      expect(MockWebSocket.lastInstance?.send).toHaveBeenCalledWith(
        JSON.stringify({
          type: 'disconnect',
          bot: 'stockfish'
        })
      );
    });
  });

  describe('sendRawCommand', () => {
    it('sends raw UCI command', async () => {
      const client = createBotClient();

      const connectPromise = client.connect();
      MockWebSocket.lastInstance?.simulateOpen();
      await connectPromise;

      MockWebSocket.lastInstance?.send.mockClear();

      client.sendRawCommand('uci');

      expect(MockWebSocket.lastInstance?.send).toHaveBeenCalledWith(
        JSON.stringify({
          type: 'uci',
          cmd: 'uci',
          bot: undefined
        })
      );
    });

    it('sends raw UCI command to specific bot', async () => {
      const client = createBotClient();

      const connectPromise = client.connect();
      MockWebSocket.lastInstance?.simulateOpen();
      await connectPromise;

      MockWebSocket.lastInstance?.send.mockClear();

      client.sendRawCommand('setoption name Hash value 256', 'stockfish');

      expect(MockWebSocket.lastInstance?.send).toHaveBeenCalledWith(
        JSON.stringify({
          type: 'uci',
          cmd: 'setoption name Hash value 256',
          bot: 'stockfish'
        })
      );
    });
  });

  describe('reconnection', () => {
    it('closes existing connection when reconnecting', async () => {
      const client = createBotClient();

      const connectPromise1 = client.connect();
      const ws1 = MockWebSocket.lastInstance;
      MockWebSocket.lastInstance?.simulateOpen();
      await connectPromise1;

      const connectPromise2 = client.connect();
      MockWebSocket.lastInstance?.simulateOpen();
      await connectPromise2;

      expect(ws1?.close).toHaveBeenCalled();
    });
  });
});
