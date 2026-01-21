<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { api } from '$lib/api';
  import { parseFen, parseUciMove, getSideToMove, STARTING_FEN } from '$lib/fen';
  import { Board } from '@tmttn-chess/board';
  import type { MatchDetail, Move, Game } from '$lib/types';

  let matchDetail: MatchDetail | null = $state(null);
  let moves: Move[] = $state([]);
  let currentPly = $state(0);
  let selectedGame = $state(0);
  let loading = $state(true);
  let error = $state<string | null>(null);

  const id = $derived($page.params.id);

  async function loadGameMoves(game: Game) {
    if (!game.id) return;
    try {
      moves = await api.getGameMoves(game.id);
      currentPly = 0;
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load moves';
    }
  }

  onMount(async () => {
    if (!id) {
      error = 'No match ID provided';
      loading = false;
      return;
    }
    try {
      matchDetail = await api.getMatch(id);
      if (matchDetail.games.length > 0) {
        await loadGameMoves(matchDetail.games[0]);
      }
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load match';
    } finally {
      loading = false;
    }
  });

  async function selectGame(index: number) {
    if (!matchDetail || index === selectedGame) return;
    selectedGame = index;
    await loadGameMoves(matchDetail.games[index]);
  }

  const currentFen = $derived(
    currentPly === 0 ? STARTING_FEN : moves[currentPly - 1]?.fen_after ?? STARTING_FEN
  );

  const board = $derived(parseFen(currentFen));
  const sideToMove = $derived(getSideToMove(currentFen));
  const lastMove = $derived(
    currentPly > 0 ? parseUciMove(moves[currentPly - 1].uci) : null
  );

  function goTo(ply: number) {
    if (ply >= 0 && ply <= moves.length) {
      currentPly = ply;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'ArrowLeft') goTo(currentPly - 1);
    if (e.key === 'ArrowRight') goTo(currentPly + 1);
    if (e.key === 'Home') goTo(0);
    if (e.key === 'End') goTo(moves.length);
  }

  function formatMoveNumber(index: number): string {
    return `${Math.floor(index / 2) + 1}.`;
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="game-detail">
  {#if loading}
    <p class="loading">Loading...</p>
  {:else if error}
    <p class="error">{error}</p>
  {:else if matchDetail}
    <header>
      <div class="title-section">
        <h1>{matchDetail.white_bot} vs {matchDetail.black_bot}</h1>
        <p class="score">{matchDetail.white_score} - {matchDetail.black_score}</p>
      </div>
      {#if matchDetail.games.length > 1}
        <div class="game-selector">
          {#each matchDetail.games as game, i}
            <button
              class="game-tab"
              class:active={selectedGame === i}
              onclick={() => selectGame(i)}
            >
              Game {game.game_number}
              {#if game.result}
                <span class="result">({game.result})</span>
              {/if}
            </button>
          {/each}
        </div>
      {/if}
    </header>

    <div class="content">
      <div class="board-section">
        <div class="board-container">
          <Board {board} legalMoves={[]} {lastMove} {sideToMove} />
        </div>
        <div class="controls">
          <button onclick={() => goTo(0)} title="Start (Home)">|&lt;</button>
          <button onclick={() => goTo(currentPly - 1)} title="Previous (Left Arrow)">&lt;</button>
          <span class="ply-info">{currentPly} / {moves.length}</span>
          <button onclick={() => goTo(currentPly + 1)} title="Next (Right Arrow)">&gt;</button>
          <button onclick={() => goTo(moves.length)} title="End (End)">&gt;|</button>
        </div>
      </div>

      <div class="moves-panel">
        <h2>Moves</h2>
        {#if matchDetail.games[selectedGame]?.opening_name}
          <p class="opening">{matchDetail.games[selectedGame].opening_name}</p>
        {/if}
        <div class="move-list">
          {#each moves as move, i}
            {#if i % 2 === 0}
              <span class="move-number">{formatMoveNumber(i)}</span>
            {/if}
            <button
              class="move"
              class:active={currentPly === i + 1}
              onclick={() => goTo(i + 1)}
            >
              {move.san ?? move.uci}
            </button>
          {/each}
        </div>
        {#if matchDetail.games[selectedGame]?.result}
          <p class="game-result">{matchDetail.games[selectedGame].result}</p>
        {/if}
      </div>
    </div>
  {/if}
</div>

<style>
  .game-detail {
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

  .score {
    font-size: 1.5rem;
    font-weight: bold;
    color: var(--highlight);
  }

  .game-selector {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
  }

  .game-tab {
    padding: 0.5rem 1rem;
    background: var(--bg-secondary);
    border: 1px solid var(--accent);
    border-radius: 4px;
    color: var(--text);
    cursor: pointer;
    transition: background 0.2s;
  }

  .game-tab:hover {
    background: var(--accent);
  }

  .game-tab.active {
    background: var(--highlight);
    border-color: var(--highlight);
  }

  .game-tab .result {
    font-size: 0.875rem;
    opacity: 0.8;
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

  .controls {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
  }

  .controls button {
    padding: 0.5rem 1rem;
    background: var(--accent);
    border: none;
    border-radius: 4px;
    color: var(--text);
    cursor: pointer;
    font-weight: bold;
    min-width: 40px;
  }

  .controls button:hover {
    background: var(--highlight);
  }

  .ply-info {
    padding: 0 1rem;
    color: var(--text-muted);
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

  .opening {
    font-style: italic;
    color: var(--text-muted);
    margin-bottom: 1rem;
    font-size: 0.875rem;
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
    background: transparent;
    border: none;
    color: var(--text);
    cursor: pointer;
    border-radius: 4px;
    font-size: 0.875rem;
  }

  .move:hover {
    background: var(--accent);
  }

  .move.active {
    background: var(--highlight);
  }

  .game-result {
    margin-top: 1rem;
    padding-top: 1rem;
    border-top: 1px solid var(--accent);
    text-align: center;
    font-weight: bold;
    color: var(--highlight);
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
