<script lang="ts">
  import { gameStore, isGameOver } from '../stores/game';
  import { isMuted, toggleMute } from '../sounds';

  interface Props {
    onflip: () => void;
  }

  let { onflip }: Props = $props();

  let canUndo = $derived($gameStore.moveHistory.length > 0);
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
  <button
    onclick={() => gameStore.undo()}
    disabled={!canUndo}
    title="Undo Move"
  >
    <span class="icon">↩</span>
    Undo
  </button>
  <button onclick={toggleMute} title={$isMuted ? 'Unmute' : 'Mute'}>
    <span class="icon">{$isMuted ? '🔇' : '🔊'}</span>
  </button>
</div>

<style>
  .controls {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
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
</style>
