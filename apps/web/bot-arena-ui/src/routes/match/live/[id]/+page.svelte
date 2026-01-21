<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { page } from '$app/stores';
  import { api } from '$lib/api';
  import { parseFen, parseUciMove, getSideToMove, STARTING_FEN } from '$lib/fen';
  import { createLiveMatchStore, type LiveMatchState } from '$lib/ws';
  import { Board } from '@tmttn-chess/board';
  import type { Match } from '$lib/types';

  let match: Match | null = $state(null);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let liveState: LiveMatchState = $state({
    connected: false,
    moves: [],
    currentGame: 1,
    score: { white: 0, black: 0 },
  });

  const id = $derived($page.params.id);

  let store: ReturnType<typeof createLiveMatchStore> | null = null;
  let unsubscribe: (() => void) | null = null;

  onMount(async () => {
    if (!id) {
      error = 'No match ID provided';
      loading = false;
      return;
    }

    try {
      match = await api.getMatch(id);
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load match';
      loading = false;
      return;
    }

    loading = false;

    // Set up WebSocket connection
    store = createLiveMatchStore(id);
    unsubscribe = store.subscribe(state => {
      liveState = state;
    });
    store.connect();
  });

  onDestroy(() => {
    if (store) {
      store.disconnect();
    }
    if (unsubscribe) {
      unsubscribe();
    }
  });

  // Compute current position from moves
  // Since we don't have FEN from WebSocket, we show starting position
  // In a real implementation, we would track FEN or compute it from moves
  const currentFen = $derived(
    liveState.moves.length > 0 && liveState.moves[liveState.moves.length - 1].fen_after
      ? liveState.moves[liveState.moves.length - 1].fen_after
      : STARTING_FEN
  );

  const board = $derived(parseFen(currentFen));
  const sideToMove = $derived(getSideToMove(currentFen));
  const lastMove = $derived(
    liveState.moves.length > 0
      ? parseUciMove(liveState.moves[liveState.moves.length - 1].uci)
      : null
  );

  function formatMoveNumber(index: number): string {
    return `${Math.floor(index / 2) + 1}.`;
  }

  function formatEval(centipawns: number | null): string {
    if (centipawns === null) return '';
    const pawns = centipawns / 100;
    return pawns >= 0 ? `+${pawns.toFixed(2)}` : pawns.toFixed(2);
  }
</script>

<div class="live-match">
  {#if loading}
    <p class="loading">Loading match...</p>
  {:else if error}
    <p class="error">{error}</p>
  {:else if match}
    <header>
      <div class="title-section">
        <h1>{match.white_bot} vs {match.black_bot}</h1>
        <div class="connection-status" class:connected={liveState.connected}>
          {liveState.connected ? 'Live' : 'Disconnected'}
        </div>
      </div>
      <div class="match-info">
        <span class="game-number">Game {liveState.currentGame}</span>
        <span class="score">{liveState.score.white} - {liveState.score.black}</span>
      </div>
    </header>

    <div class="content">
      <div class="board-section">
        <div class="board-container">
          <Board {board} legalMoves={[]} {lastMove} {sideToMove} />
        </div>
      </div>

      <div class="moves-panel">
        <h2>Moves</h2>
        {#if liveState.moves.length === 0}
          <p class="waiting">Waiting for moves...</p>
        {:else}
          <div class="move-list">
            {#each liveState.moves as move, i}
              {#if i % 2 === 0}
                <span class="move-number">{formatMoveNumber(i)}</span>
              {/if}
              <span class="move">
                {move.san ?? move.uci}
                {#if move.bot_eval !== null}
                  <span class="eval">{formatEval(move.bot_eval)}</span>
                {/if}
              </span>
            {/each}
          </div>
        {/if}
      </div>
    </div>
  {/if}
</div>

<style>
  .live-match {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  header {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .title-section {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  h1 {
    font-size: 1.5rem;
  }

  .connection-status {
    padding: 0.25rem 0.75rem;
    border-radius: 9999px;
    font-size: 0.875rem;
    font-weight: 500;
    background: var(--accent);
    color: var(--text-muted);
  }

  .connection-status.connected {
    background: var(--success);
    color: var(--bg);
  }

  .match-info {
    display: flex;
    gap: 2rem;
    align-items: center;
  }

  .game-number {
    font-size: 1rem;
    color: var(--text-muted);
  }

  .score {
    font-size: 1.5rem;
    font-weight: bold;
    color: var(--highlight);
  }

  .content {
    display: grid;
    grid-template-columns: auto 300px;
    gap: 2rem;
    align-items: start;
  }

  .board-section {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .board-container {
    --square-size: 70px;
    padding-left: 1.25rem;
    padding-bottom: 1.5rem;
  }

  .moves-panel {
    background: var(--bg-secondary);
    border-radius: 8px;
    padding: 1rem;
  }

  .moves-panel h2 {
    margin-bottom: 0.5rem;
    font-size: 1rem;
  }

  .waiting {
    color: var(--text-muted);
    font-style: italic;
  }

  .move-list {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: 0.25rem 0.125rem;
    max-height: 400px;
    overflow-y: auto;
  }

  .move-number {
    width: 2.5rem;
    color: var(--text-muted);
    font-size: 0.875rem;
  }

  .move {
    padding: 0.25rem 0.5rem;
    color: var(--text);
    border-radius: 4px;
    font-size: 0.875rem;
  }

  .eval {
    font-size: 0.75rem;
    color: var(--text-muted);
    margin-left: 0.25rem;
  }

  .loading, .error {
    text-align: center;
    padding: 2rem;
  }

  .error {
    color: var(--highlight);
  }

  @media (max-width: 800px) {
    .content {
      grid-template-columns: 1fr;
    }

    .board-container {
      --square-size: min(10vw, 60px);
    }
  }
</style>
