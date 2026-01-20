# @chess/bot-client

WebSocket client for communicating with UCI chess engines via bot-bridge.

## Installation

```bash
npm install @chess/bot-client
```

## Usage

```typescript
import { createBotClient } from '@chess/bot-client';

const client = createBotClient({
  url: 'ws://127.0.0.1:9999',
  onBestMove: (move, searchInfo) => {
    console.log('Best move:', move);
    if (searchInfo) {
      console.log('Depth:', searchInfo.depth, 'Score:', searchInfo.score);
    }
  },
  onSearchInfo: (info) => {
    console.log('Searching depth', info.depth);
  }
});

// Connect to bot-bridge
await client.connect();

// Start a session with a specific bot
const session = await client.startSession('stockfish');

// Send position and search
session.sendPosition(['e2e4', 'e7e5']);
session.go({ movetime: 1000 });

// Stop search
session.stop();

// Clean up
session.close();
client.disconnect();
```

## API

### `createBotClient(config?: BotClientConfig): BotClient`

Creates a new bot client instance.

#### BotClientConfig

```typescript
interface BotClientConfig {
  url?: string;                                    // Default: 'ws://127.0.0.1:9999'
  onConnect?: () => void;
  onDisconnect?: () => void;
  onError?: (message: string) => void;
  onBots?: (bots: string[]) => void;
  onSearchInfo?: (info: SearchInfo) => void;
  onBestMove?: (move: string, searchInfo: SearchInfo | null) => void;
}
```

### BotClient

#### State Stores

| Store | Type | Description |
|-------|------|-------------|
| `connected` | `Readable<boolean>` | Connection status |
| `connecting` | `Readable<boolean>` | Whether currently connecting |
| `availableBots` | `Readable<string[]>` | List of available bots |
| `searchInfo` | `Readable<SearchInfo \| null>` | Current search info |
| `error` | `Readable<string \| null>` | Last error message |

#### Methods

| Method | Returns | Description |
|--------|---------|-------------|
| `connect()` | `Promise<void>` | Connect to bot-bridge server |
| `disconnect()` | `void` | Disconnect from server |
| `startSession(botName)` | `Promise<BotSession>` | Start session with a bot |
| `sendRawCommand(cmd, bot?)` | `void` | Send raw UCI command |

### BotSession

A session represents an active connection to a specific bot.

```typescript
interface BotSession {
  name: string;
  sendPosition(moves: string[]): void;
  go(options: GoOptions): void;
  stop(): void;
  close(): void;
}
```

#### Session Methods

| Method | Description |
|--------|-------------|
| `sendPosition(moves)` | Set position from startpos with moves |
| `go(options)` | Start engine search |
| `stop()` | Stop current search |
| `close()` | Close this session |

### GoOptions

```typescript
interface GoOptions {
  movetime?: number;   // Search for N milliseconds
  depth?: number;      // Search to depth N
  infinite?: boolean;  // Search until stopped
}
```

### SearchInfo

Engine search information received during analysis.

```typescript
interface SearchInfo {
  depth: number;    // Search depth
  score: number;    // Score in centipawns (or mate score)
  nodes: number;    // Nodes searched
  time: number;     // Time in milliseconds
  pv: string[];     // Principal variation (best line)
}
```

## Example with Svelte

```svelte
<script lang="ts">
  import { createBotClient } from '@chess/bot-client';

  const client = createBotClient({
    onBestMove: (move) => handleBotMove(move)
  });

  let session: BotSession | null = null;

  async function connectToBot() {
    await client.connect();
    session = await client.startSession('stockfish');
  }

  function requestMove(moves: string[]) {
    if (session) {
      session.sendPosition(moves);
      session.go({ movetime: 500 });
    }
  }
</script>

{#if $client.connected}
  <p>Connected! Bots: {$client.availableBots.join(', ')}</p>
{:else}
  <button onclick={connectToBot}>Connect</button>
{/if}
```

## TypeScript Types

```typescript
import type {
  SearchInfo,
  BotClientCallbacks,
  BotClientConfig,
  BotSession,
  GoOptions,
  BotClient
} from '@chess/bot-client';
```
