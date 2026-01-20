<script lang="ts">
  import { gameStore, legalMoves, isCheck, isGameOver, sideToMove, isViewingHistory, viewFen, liveFen } from '../stores/game';
  import { loadFen, getLegalMoves, type Game } from '../wasm';

  let fenInput = $state('');
  let fenError = $state('');

  function loadFenInput() {
    if (gameStore.loadFen(fenInput)) {
      fenError = '';
    } else {
      fenError = 'Invalid FEN string';
    }
  }

  function copyFen() {
    navigator.clipboard.writeText(displayFen);
  }

  function playMove(uci: string) {
    // If viewing history, go to live first
    if ($isViewingHistory) {
      gameStore.goToLive();
    }
    gameStore.makeMove(uci);
  }

  // Sync input with displayed FEN
  $effect(() => {
    fenInput = displayFen;
  });

  // Show viewed FEN when in history, live FEN otherwise
  const displayFen = $derived($isViewingHistory ? $viewFen : $liveFen);

  // Parse FEN to get turn info for viewed position
  const viewTurn = $derived.by(() => {
    const parts = displayFen.split(' ');
    return parts[1] === 'w' ? 'white' : 'black';
  });

  // Get legal moves for viewed position (or live if not viewing history)
  const displayMoves = $derived.by(() => {
    if (!$isViewingHistory) {
      return $legalMoves;
    }
    // Load viewed position to get legal moves
    const tempGame = loadFen(displayFen);
    if (!tempGame) return [];
    return getLegalMoves(tempGame);
  });

  // Status for viewed position
  const viewStatus = $derived.by(() => {
    if (!$isViewingHistory) {
      if ($gameStore.result) {
        const r = $gameStore.result;
        if (r === 'white_wins') return 'Checkmate - White wins';
        if (r === 'black_wins') return 'Checkmate - Black wins';
        if (r === 'draw') return 'Draw';
        return r;
      }
      if ($isCheck) return 'Check!';
      return 'In progress';
    }
    // For history view, just show basic status
    return 'In progress';
  });

  const turnText = $derived(viewTurn === 'white' ? 'White to move' : 'Black to move');
</script>

<div class="debug-panel">
  <h3>Debug</h3>

  {#if $isViewingHistory}
    <div class="history-notice">
      Viewing historical position
    </div>
  {/if}

  <section class="fen-section">
    <label for="fen-input">FEN {$isViewingHistory ? '(viewed)' : ''}</label>
    <div class="fen-row">
      <input
        id="fen-input"
        type="text"
        bind:value={fenInput}
        class="monospace"
        placeholder="Enter FEN..."
      />
      <button onclick={loadFenInput}>Load</button>
      <button onclick={copyFen}>Copy</button>
    </div>
    {#if fenError}
      <p class="error">{fenError}</p>
    {/if}
  </section>

  <section class="state-section">
    <h4>Game State {$isViewingHistory ? '(viewed)' : ''}</h4>
    <dl>
      <dt>Turn</dt>
      <dd>
        <span class="dot {viewTurn}"></span>
        {turnText}
      </dd>

      <dt>Status</dt>
      <dd class:check={!$isViewingHistory && $isCheck} class:game-over={!$isViewingHistory && $isGameOver}>{viewStatus}</dd>

      <dt>Halfmove Clock</dt>
      <dd class="monospace">{displayFen.split(' ')[4] ?? '0'}</dd>

      <dt>Fullmove</dt>
      <dd class="monospace">{displayFen.split(' ')[5] ?? '1'}</dd>
    </dl>
  </section>

  <section class="moves-section">
    <h4>Legal Moves ({displayMoves.length}) {$isViewingHistory ? '(viewed)' : ''}</h4>
    <div class="moves-grid">
      {#each displayMoves as move}
        <button class="move-btn monospace" onclick={() => playMove(move.uci)}>
          {move.uci}
        </button>
      {/each}
    </div>
  </section>
</div>

<style>
  .debug-panel {
    background: var(--bg-secondary);
    border-radius: var(--radius);
    padding: 1rem;
    font-size: 0.875rem;
  }

  h3 {
    font-size: 0.875rem;
    font-weight: 600;
    margin-bottom: 1rem;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  h4 {
    font-size: 0.75rem;
    font-weight: 600;
    margin-bottom: 0.5rem;
    color: var(--text-muted);
    text-transform: uppercase;
  }

  .history-notice {
    background: var(--accent);
    color: white;
    padding: 0.5rem 0.75rem;
    border-radius: var(--radius);
    font-size: 0.75rem;
    margin-bottom: 1rem;
    text-align: center;
  }

  section {
    margin-bottom: 1rem;
  }

  section:last-child {
    margin-bottom: 0;
  }

  label {
    display: block;
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--text-muted);
    text-transform: uppercase;
    margin-bottom: 0.25rem;
  }

  .fen-row {
    display: flex;
    gap: 0.5rem;
  }

  .fen-row input {
    flex: 1;
    font-size: 0.75rem;
  }

  .fen-row button {
    padding: 0.375rem 0.625rem;
    font-size: 0.75rem;
  }

  .error {
    color: var(--error);
    font-size: 0.75rem;
    margin-top: 0.25rem;
  }

  dl {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 0.25rem 0.75rem;
  }

  dt {
    color: var(--text-muted);
  }

  dd {
    display: flex;
    align-items: center;
    gap: 0.375rem;
  }

  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
  }

  .dot.white {
    background: #fff;
    border: 1px solid #999;
  }

  .dot.black {
    background: #333;
  }

  .check {
    color: var(--warning);
    font-weight: 600;
  }

  .game-over {
    color: var(--accent);
    font-weight: 600;
  }

  .moves-grid {
    display: flex;
    flex-wrap: wrap;
    gap: 0.25rem;
    max-height: 150px;
    overflow-y: auto;
  }

  .move-btn {
    padding: 0.25rem 0.5rem;
    font-size: 0.75rem;
    background: var(--bg-tertiary);
  }

  .move-btn:hover {
    background: var(--accent);
    color: #fff;
  }
</style>
