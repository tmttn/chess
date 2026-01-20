<script lang="ts">
  import { gameStore, isGameOver, moveHistory, viewIndex } from '../stores/game';

  let showUci = $state(false);

  // Group moves into pairs for display
  const movePairs = $derived.by(() => {
    const history = $moveHistory;
    const pairs: { number: number; white: { san: string; uci: string; index: number } | null; black: { san: string; uci: string; index: number } | null }[] = [];

    for (let i = 0; i < history.length; i += 2) {
      pairs.push({
        number: Math.floor(i / 2) + 1,
        white: history[i] ? { san: history[i].san, uci: history[i].uci, index: i } : null,
        black: history[i + 1] ? { san: history[i + 1].san, uci: history[i + 1].uci, index: i + 1 } : null
      });
    }

    return pairs;
  });

  // Derive result text from move history - last move's FEN indicates result
  const resultText = $derived.by(() => {
    if (!$isGameOver) return null;
    const history = $moveHistory;
    if (history.length === 0) return null;
    // Check the last FEN for result indicator
    const lastFen = history[history.length - 1]?.fen ?? '';
    // For now, we'll determine result from game state
    // A proper implementation would check the actual result from the game
    // but for display purposes, we can show generic result when game is over
    return 'Game Over';
  });

  function handleMoveClick(index: number) {
    gameStore.goToMove(index);
  }

  function getMoveText(move: { san: string; uci: string }): string {
    return showUci ? move.uci : move.san;
  }
</script>

<div class="move-history">
  <div class="header">
    <h3>Moves</h3>
    <button class="notation-toggle" onclick={() => showUci = !showUci} title="Toggle notation">
      {showUci ? 'UCI' : 'SAN'}
    </button>
  </div>
  <div class="moves-list">
    {#if movePairs.length === 0}
      <p class="empty">No moves yet</p>
    {:else}
      {#each movePairs as pair}
        <div class="move-row">
          <span class="move-number">{pair.number}.</span>
          {#if pair.white}
            <button
              class="move white"
              class:active={$viewIndex === pair.white.index}
              onclick={() => handleMoveClick(pair.white!.index)}
            >
              {getMoveText(pair.white)}
            </button>
          {:else}
            <span class="move white"></span>
          {/if}
          {#if pair.black}
            <button
              class="move black"
              class:active={$viewIndex === pair.black.index}
              onclick={() => handleMoveClick(pair.black!.index)}
            >
              {getMoveText(pair.black)}
            </button>
          {:else}
            <span class="move black"></span>
          {/if}
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

  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 0.75rem;
  }

  h3 {
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin: 0;
  }

  .notation-toggle {
    font-size: 0.625rem;
    padding: 0.25rem 0.5rem;
    background: var(--bg-tertiary);
    border: none;
    font-family: 'SF Mono', Monaco, monospace;
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
    text-align: left;
    background: transparent;
    border: none;
    color: var(--text-primary);
    font: inherit;
    cursor: pointer;
  }

  .move:hover {
    background: var(--bg-tertiary);
  }

  .move.active {
    background: var(--accent);
    color: #fff;
  }

  .result {
    text-align: center;
    font-weight: 600;
    margin-top: 0.5rem;
    padding-top: 0.5rem;
    border-top: 1px solid var(--bg-tertiary);
  }
</style>
