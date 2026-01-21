<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '$lib/api';
  import type { Match } from '$lib/types';

  let matches: Match[] = $state([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let offset = $state(0);
  const limit = 20;

  async function loadMatches() {
    loading = true;
    error = null;
    try {
      matches = await api.getMatches({ limit, offset });
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load matches';
    } finally {
      loading = false;
    }
  }

  onMount(loadMatches);

  function prevPage() {
    if (offset >= limit) {
      offset -= limit;
      loadMatches();
    }
  }

  function nextPage() {
    if (matches.length === limit) {
      offset += limit;
      loadMatches();
    }
  }

  function formatResult(match: Match): string {
    if (match.white_score > match.black_score) return 'White wins';
    if (match.black_score > match.white_score) return 'Black wins';
    return 'Draw';
  }
</script>

<div class="games-page">
  <h1>Game Browser</h1>

  {#if loading}
    <p class="loading">Loading...</p>
  {:else if error}
    <p class="error">{error}</p>
  {:else}
    <table>
      <thead>
        <tr>
          <th>Date</th>
          <th>White</th>
          <th>Black</th>
          <th>Score</th>
          <th>Games</th>
          <th>Result</th>
        </tr>
      </thead>
      <tbody>
        {#each matches as match}
          <tr>
            <td>{new Date(match.started_at).toLocaleDateString()}</td>
            <td>{match.white_bot}</td>
            <td>{match.black_bot}</td>
            <td>{match.white_score}-{match.black_score}</td>
            <td>{match.games_total}</td>
            <td>
              <a href="/games/{match.id}">{formatResult(match)}</a>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>

    <div class="pagination">
      <button onclick={prevPage} disabled={offset === 0}>Previous</button>
      <span>Page {Math.floor(offset / limit) + 1}</span>
      <button onclick={nextPage} disabled={matches.length < limit}>Next</button>
    </div>
  {/if}
</div>

<style>
  .games-page {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  table {
    width: 100%;
    border-collapse: collapse;
    background: var(--bg-secondary);
    border-radius: 8px;
    overflow: hidden;
  }

  th, td {
    padding: 1rem;
    text-align: left;
    border-bottom: 1px solid var(--accent);
  }

  th {
    background: var(--accent);
  }

  .pagination {
    display: flex;
    justify-content: center;
    align-items: center;
    gap: 1rem;
  }

  button {
    padding: 0.5rem 1rem;
    background: var(--accent);
    border: none;
    border-radius: 4px;
    color: var(--text);
    cursor: pointer;
  }

  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .loading, .error {
    text-align: center;
    padding: 2rem;
  }

  .error {
    color: var(--highlight);
  }
</style>
