<script lang="ts">
  import { gameStore, isViewingHistory, viewIndex, moveHistory } from '../stores/game';
  import { isMuted, toggleMute } from '@tmttn-chess/game-store';

  interface Props {
    onflip: () => void;
  }

  let { onflip }: Props = $props();

  let canGoPrev = $derived($viewIndex >= 0);
  let canGoNext = $derived($viewIndex < $moveHistory.length - 1);
  let viewing = $derived($isViewingHistory);
  let totalMoves = $derived($moveHistory.length);
</script>

<div class="controls">
  <button onclick={() => gameStore.newGame()} title="New Game">
    <span class="icon">↻</span>
    New
  </button>
  <button onclick={onflip} title="Flip Board">
    <span class="icon">⇅</span>
    Flip
  </button>

  <div class="nav-group">
    <button
      onclick={() => gameStore.goToStart()}
      disabled={!canGoPrev}
      title="Go to start"
      class="nav-btn"
    >⏮</button>
    <button
      onclick={() => gameStore.viewPrev()}
      disabled={!canGoPrev}
      title="Previous move"
      class="nav-btn"
    >◀</button>
    <button
      onclick={() => gameStore.viewNext()}
      disabled={!canGoNext}
      title="Next move"
      class="nav-btn"
    >▶</button>
    <button
      onclick={() => gameStore.goToLive()}
      disabled={!canGoNext}
      title="Go to live position"
      class="nav-btn"
    >⏭</button>
  </div>

  <button onclick={toggleMute} title={$isMuted ? 'Unmute' : 'Mute'}>
    <span class="icon">{$isMuted ? '🔇' : '🔊'}</span>
  </button>
</div>

{#if viewing}
  <div class="viewing-indicator">
    <span>Viewing move {$viewIndex + 1} of {totalMoves}</span>
    <button onclick={() => gameStore.goToLive()} class="go-live-btn">Go to live</button>
  </div>
{/if}

<style>
  .controls {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
    align-items: center;
  }

  button {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    padding: 0.5rem 0.75rem;
    font-size: 0.875rem;
  }

  .icon {
    font-size: 1rem;
  }

  .nav-group {
    display: flex;
    gap: 0.125rem;
    background: var(--bg-tertiary);
    border-radius: var(--radius);
    padding: 0.125rem;
  }

  .nav-btn {
    padding: 0.375rem 0.5rem;
    min-width: 2rem;
    justify-content: center;
    background: transparent;
    border: none;
    border-radius: calc(var(--radius) - 2px);
  }

  .nav-btn:hover:not(:disabled) {
    background: var(--bg-secondary);
  }

  .nav-btn:disabled {
    opacity: 0.3;
  }

  .viewing-indicator {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    margin-top: 0.5rem;
    padding: 0.5rem 0.75rem;
    background: var(--accent);
    color: white;
    border-radius: var(--radius);
    font-size: 0.8rem;
  }

  .go-live-btn {
    padding: 0.25rem 0.5rem;
    font-size: 0.75rem;
    background: rgba(255, 255, 255, 0.2);
    border: 1px solid rgba(255, 255, 255, 0.3);
    color: white;
    border-radius: 4px;
  }

  .go-live-btn:hover {
    background: rgba(255, 255, 255, 0.3);
  }
</style>
