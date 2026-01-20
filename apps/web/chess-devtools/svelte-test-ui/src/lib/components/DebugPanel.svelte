<script lang="ts">
  import { gameStore, legalMoves, isCheck, isGameOver, sideToMove } from '../stores/game';

  let fenInput = $state('');
  let fenError = $state('');

  function loadFen() {
    if (gameStore.loadFen(fenInput)) {
      fenError = '';
    } else {
      fenError = 'Invalid FEN string';
    }
  }

  function copyFen() {
    navigator.clipboard.writeText($gameStore.fen);
  }

  function playMove(uci: string) {
    gameStore.makeMove(uci);
  }

  // Sync input with current FEN
  $effect(() => {
    fenInput = $gameStore.fen;
  });

  const statusText = $derived.by(() => {
    if ($gameStore.result) {
      const r = $gameStore.result;
      if (r === 'white_wins') return 'Checkmate - White wins';
      if (r === 'black_wins') return 'Checkmate - Black wins';
      if (r === 'draw') return 'Draw';
      return r;
    }
    if ($isCheck) return 'Check!';
    return 'In progress';
  });

  const turnText = $derived($sideToMove === 'white' ? 'White to move' : 'Black to move');
</script>

<div class="debug-panel">
  <h3>Debug</h3>

  <section class="fen-section">
    <label for="fen-input">FEN</label>
    <div class="fen-row">
      <input
        id="fen-input"
        type="text"
        bind:value={fenInput}
        class="monospace"
        placeholder="Enter FEN..."
      />
      <button onclick={loadFen}>Load</button>
      <button onclick={copyFen}>Copy</button>
    </div>
    {#if fenError}
      <p class="error">{fenError}</p>
    {/if}
  </section>

  <section class="state-section">
    <h4>Game State</h4>
    <dl>
      <dt>Turn</dt>
      <dd>
        <span class="dot {$sideToMove}"></span>
        {turnText}
      </dd>

      <dt>Status</dt>
      <dd class:check={$isCheck} class:game-over={$isGameOver}>{statusText}</dd>

      <dt>Halfmove Clock</dt>
      <dd class="monospace">{$gameStore.fen.split(' ')[4] ?? '0'}</dd>

      <dt>Fullmove</dt>
      <dd class="monospace">{$gameStore.fen.split(' ')[5] ?? '1'}</dd>
    </dl>
  </section>

  <section class="moves-section">
    <h4>Legal Moves ({$legalMoves.length})</h4>
    <div class="moves-grid">
      {#each $legalMoves as move}
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
