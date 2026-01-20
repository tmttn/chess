<script lang="ts">
  interface Props {
    color: 'white' | 'black';
    onselect: (piece: 'q' | 'r' | 'b' | 'n') => void;
    oncancel: () => void;
  }

  let { color, onselect, oncancel }: Props = $props();

  const PIECE_SET = 'cburnett';
  const pieces: Array<{ type: 'q' | 'r' | 'b' | 'n'; label: string }> = [
    { type: 'q', label: 'Queen' },
    { type: 'r', label: 'Rook' },
    { type: 'b', label: 'Bishop' },
    { type: 'n', label: 'Knight' }
  ];

  function getPieceSrc(type: string): string {
    const colorPrefix = color === 'white' ? 'w' : 'b';
    return `https://lichess1.org/assets/piece/${PIECE_SET}/${colorPrefix}${type.toUpperCase()}.svg`;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      oncancel();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="overlay" onclick={oncancel} role="button" tabindex="-1">
  <div class="dialog" onclick={(e) => e.stopPropagation()} role="dialog" aria-label="Choose promotion piece">
    <div class="title">Promote to:</div>
    <div class="pieces">
      {#each pieces as piece}
        <button
          onclick={() => onselect(piece.type)}
          title={piece.label}
          class="piece-button"
        >
          <img src={getPieceSrc(piece.type)} alt={piece.label} />
        </button>
      {/each}
    </div>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }

  .dialog {
    background: var(--bg-secondary);
    border: 1px solid var(--bg-tertiary);
    border-radius: var(--radius);
    padding: 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .title {
    font-size: 0.875rem;
    color: var(--text-secondary);
    text-align: center;
  }

  .pieces {
    display: flex;
    gap: 0.5rem;
  }

  .piece-button {
    width: 60px;
    height: 60px;
    padding: 0.25rem;
    background: var(--bg-tertiary);
    border: 2px solid transparent;
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: border-color 0.15s, transform 0.1s;
  }

  .piece-button:hover {
    border-color: var(--accent);
    transform: scale(1.05);
  }

  .piece-button img {
    width: 100%;
    height: 100%;
  }
</style>
