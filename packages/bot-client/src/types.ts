// packages/bot-client/src/types.ts

import type { Readable } from 'svelte/store';

/** Search information from UCI engine */
export interface SearchInfo {
  depth: number;
  score: number;
  nodes: number;
  time: number;
  pv: string[];
}

/** Event callbacks for the bot client */
export interface BotClientCallbacks {
  onConnect?: () => void;
  onDisconnect?: () => void;
  onError?: (message: string) => void;
  onBots?: (bots: string[]) => void;
  onSearchInfo?: (info: SearchInfo) => void;
  onBestMove?: (move: string, searchInfo: SearchInfo | null) => void;
}

/** Configuration for creating a bot client */
export interface BotClientConfig extends BotClientCallbacks {
  url?: string;
}

/** A session with a specific bot */
export interface BotSession {
  name: string;
  sendPosition(moves: string[]): void;
  go(options: GoOptions): void;
  stop(): void;
  close(): void;
}

/** Options for the "go" command */
export interface GoOptions {
  movetime?: number;
  depth?: number;
  infinite?: boolean;
}

/** The public bot client interface */
export interface BotClient {
  // State stores
  connected: Readable<boolean>;
  connecting: Readable<boolean>;
  availableBots: Readable<string[]>;
  searchInfo: Readable<SearchInfo | null>;
  error: Readable<string | null>;

  // Connection methods
  connect(): Promise<void>;
  disconnect(): void;

  // Session management
  startSession(botName: string): Promise<BotSession>;

  // Direct commands (for debugging)
  sendRawCommand(cmd: string, bot?: string): void;
}
