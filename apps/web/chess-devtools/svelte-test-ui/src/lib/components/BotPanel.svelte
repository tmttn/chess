<script lang="ts">
  import { botStore, isConnected, availableBots } from '$lib/stores/bot';
  import { sideToMove, isGameOver } from '$lib/stores/game';

  let connecting = $derived($botStore.connecting);
  let connected = $derived($isConnected);
  let bots = $derived($availableBots);
  let whitePlayer = $derived($botStore.whitePlayer);
  let blackPlayer = $derived($botStore.blackPlayer);
  let autoPlay = $derived($botStore.autoPlay);
  let error = $derived($botStore.error);

  function handleConnect() {
    botStore.connect();
  }

  function handleDisconnect() {
    botStore.disconnect();
  }

  function handleWhitePlayerChange(e: Event) {
    const select = e.target as HTMLSelectElement;
    botStore.setWhitePlayer(select.value);
  }

  function handleBlackPlayerChange(e: Event) {
    const select = e.target as HTMLSelectElement;
    botStore.setBlackPlayer(select.value);
  }
</script>

<div class="bot-panel">
  <h3>Bot Control</h3>

  {#if error}
    <div class="error">
      {error}
      <button onclick={() => botStore.clearError()}>Dismiss</button>
    </div>
  {/if}

  <div class="connection-section">
    {#if !connected && !connecting}
      <button onclick={handleConnect} class="connect-btn">
        Connect to Bridge
      </button>
    {:else if connecting}
      <button disabled class="connect-btn">
        Connecting...
      </button>
    {:else}
      <div class="connected-status">
        <span class="status-dot"></span>
        Connected
        <button onclick={handleDisconnect} class="disconnect-btn">
          Disconnect
        </button>
      </div>
    {/if}
  </div>

  {#if connected && bots.length > 0}
    <div class="player-assignment">
      <div class="player-row">
        <span class="player-label white-label">White:</span>
        <select onchange={handleWhitePlayerChange} value={whitePlayer}>
          <option value="human">Human</option>
          {#each bots as bot}
            <option value={bot}>{bot}</option>
          {/each}
        </select>
      </div>

      <div class="player-row">
        <span class="player-label black-label">Black:</span>
        <select onchange={handleBlackPlayerChange} value={blackPlayer}>
          <option value="human">Human</option>
          {#each bots as bot}
            <option value={bot}>{bot}</option>
          {/each}
        </select>
      </div>
    </div>

    <div class="auto-play-section">
      <label class="auto-play-toggle">
        <input
          type="checkbox"
          checked={autoPlay}
          onchange={() => botStore.toggleAutoPlay()}
        />
        Auto-play enabled
      </label>

      {#if autoPlay && !$isGameOver}
        <div class="turn-indicator">
          {$sideToMove === 'white' ? 'White' : 'Black'} to move
          {#if ($sideToMove === 'white' && whitePlayer !== 'human') || ($sideToMove === 'black' && blackPlayer !== 'human')}
            <span class="thinking">(bot thinking...)</span>
          {/if}
        </div>
      {/if}
    </div>
  {:else if connected}
    <div class="no-bots">No bots available</div>
  {/if}
</div>

<style>
  .bot-panel {
    background: var(--bg-secondary);
    border-radius: 8px;
    padding: 1rem;
  }

  h3 {
    margin: 0 0 0.75rem 0;
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  .error {
    background: #ff4444;
    color: white;
    padding: 0.5rem;
    border-radius: 4px;
    margin-bottom: 0.75rem;
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 0.8rem;
  }

  .error button {
    background: transparent;
    border: 1px solid white;
    color: white;
    padding: 0.25rem 0.5rem;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.75rem;
  }

  .connection-section {
    margin-bottom: 0.75rem;
  }

  .connect-btn {
    width: 100%;
    padding: 0.5rem;
    background: var(--accent);
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.875rem;
  }

  .connect-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .connected-status {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.875rem;
  }

  .status-dot {
    width: 8px;
    height: 8px;
    background: #4caf50;
    border-radius: 50%;
  }

  .disconnect-btn {
    margin-left: auto;
    background: transparent;
    border: 1px solid var(--text-muted);
    color: var(--text-muted);
    padding: 0.25rem 0.5rem;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.75rem;
  }

  .disconnect-btn:hover {
    border-color: var(--text-primary);
    color: var(--text-primary);
  }

  select {
    padding: 0.5rem;
    border-radius: 4px;
    border: 1px solid var(--bg-tertiary);
    background: var(--bg-primary);
    color: var(--text-primary);
    font-size: 0.875rem;
  }

  .player-assignment {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    margin-bottom: 0.75rem;
  }

  .player-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .player-label {
    width: 50px;
    font-size: 0.8rem;
    font-weight: 500;
  }

  .white-label {
    color: #f0f0f0;
  }

  .black-label {
    color: #999;
  }

  .player-row select {
    flex: 1;
  }

  .auto-play-section {
    padding-top: 0.5rem;
    border-top: 1px solid var(--bg-tertiary);
  }

  .auto-play-toggle {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.875rem;
    cursor: pointer;
  }

  .turn-indicator {
    margin-top: 0.5rem;
    font-size: 0.8rem;
    color: var(--text-muted);
  }

  .thinking {
    color: var(--accent);
    animation: pulse 1s infinite;
  }

  .no-bots {
    color: var(--text-muted);
    font-size: 0.8rem;
    font-style: italic;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }
</style>
