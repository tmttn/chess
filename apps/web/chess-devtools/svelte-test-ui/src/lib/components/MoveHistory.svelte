<script lang="ts">
  import { gameStore, isGameOver } from '../stores/game';

  // Group moves into pairs for display
  const movePairs = $derived.by(() => {
    const history = $gameStore.moveHistory;
    const pairs: { number: number; white: string; black: string | null }[] = [];

    for (let i = 0; i < history.length; i += 2) {
      pairs.push({
        number: Math.floor(i / 2) + 1,
        white: history[i]?.uci ?? '',
        black: history[i + 1]?.uci ?? null
      });
    }

    return pairs;
  });

  const resultText = $derived.by(() => {
    const result = $gameStore.result;
    if (!result) return null;
    if (result === 'white_wins') return '1-0';
    if (result === 'black_wins') return '0-1';
    return '½-½';
  });
</script>

<div class="move-history">
  <h3>Moves</h3>
  <div class="moves-list">
    {#if movePairs.length === 0}
      <p class="empty">No moves yet</p>
    {:else}
      {#each movePairs as pair}
        <div class="move-row">
          <span class="move-number">{pair.number}.</span>
          <span class="move white">{pair.white}</span>
          <span class="move black">{pair.black ?? ''}</span>
        </div>
      {/each}
      {#if resultText}
        <div class="result">{resultText}</div>
      {/if}
    {/if}
  </div>
</div>

<style>
  .move-history {
    background: var(--bg-secondary);
    border-radius: var(--radius);
    padding: 1rem;
    min-width: 180px;
  }

  h3 {
    font-size: 0.875rem;
    font-weight: 600;
    margin-bottom: 0.75rem;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .moves-list {
    max-height: 200px;
    overflow-y: auto;
    font-family: 'SF Mono', Monaco, monospace;
    font-size: 0.875rem;
  }

  .empty {
    color: var(--text-muted);
    font-style: italic;
  }

  .move-row {
    display: grid;
    grid-template-columns: 2rem 1fr 1fr;
    gap: 0.25rem;
    padding: 0.125rem 0;
  }

  .move-number {
    color: var(--text-muted);
  }

  .move {
    padding: 0.125rem 0.25rem;
    border-radius: 2px;
  }

  .move:hover {
    background: var(--bg-tertiary);
  }

  .result {
    text-align: center;
    font-weight: 600;
    margin-top: 0.5rem;
    padding-top: 0.5rem;
    border-top: 1px solid var(--bg-tertiary);
  }
</style>
