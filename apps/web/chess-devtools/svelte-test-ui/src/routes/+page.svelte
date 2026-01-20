<script lang="ts">
  import Board from '$lib/components/Board.svelte';
  import GameControls from '$lib/components/GameControls.svelte';
  import MoveHistory from '$lib/components/MoveHistory.svelte';
  import DebugPanel from '$lib/components/DebugPanel.svelte';

  let flipped = $state(false);
</script>

<main>
  <header>
    <h1>Chess Devtools</h1>
    <span class="subtitle">Test UI</span>
  </header>

  <div class="game-container">
    <div class="controls-bar">
      <GameControls onflip={() => flipped = !flipped} />
    </div>

    <div class="main-content">
      <div class="board-area">
        <Board {flipped} />
      </div>

      <div class="side-panel">
        <DebugPanel />
        <MoveHistory />
      </div>
    </div>
  </div>
</main>

<style>
  main {
    max-width: 1200px;
    margin: 0 auto;
    padding: 2rem;
  }

  header {
    display: flex;
    align-items: baseline;
    gap: 0.75rem;
    margin-bottom: 1.5rem;
  }

  h1 {
    font-size: 1.5rem;
    font-weight: 600;
  }

  .subtitle {
    color: var(--text-muted);
    font-size: 0.875rem;
  }

  .game-container {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .controls-bar {
    padding-bottom: 0.5rem;
    border-bottom: 1px solid var(--bg-tertiary);
  }

  .main-content {
    display: flex;
    gap: 1.5rem;
    align-items: flex-start;
  }

  .board-area {
    padding: 1.5rem 0 1.5rem 1.5rem;
  }

  .side-panel {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    min-width: 280px;
  }

  @media (max-width: 800px) {
    .main-content {
      flex-direction: column;
    }

    .board-area {
      padding: 1rem;
    }

    .side-panel {
      width: 100%;
    }
  }
</style>
