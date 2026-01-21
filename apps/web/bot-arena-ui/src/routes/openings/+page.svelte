<script lang="ts">
  import { browser } from '$app/environment';
  import { api, type OpeningStats } from '$lib/api';

  let openings: OpeningStats[] = $state([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let searchQuery = $state('');

  /** Filter openings by search query (matches ECO code or name) */
  let filteredOpenings = $derived(
    openings.filter((opening) => {
      const query = searchQuery.toLowerCase();
      return (
        opening.eco.toLowerCase().includes(query) ||
        opening.name.toLowerCase().includes(query)
      );
    })
  );

  $effect(() => {
    if (!browser) return;

    async function loadData() {
      try {
        openings = await api.getOpenings();
      } catch (e) {
        error = e instanceof Error ? e.message : 'Failed to load openings';
      } finally {
        loading = false;
      }
    }

    loadData();
  });

  /**
   * Calculate percentage from count and total
   * @param count - Number of occurrences
   * @param total - Total number of games
   * @returns Formatted percentage string
   */
  function percentage(count: number, total: number): string {
    if (total === 0) return '-';
    return ((count / total) * 100).toFixed(1) + '%';
  }
</script>

<div class="openings-page">
  <h1>Opening Explorer</h1>

  <div class="search-container">
    <input
      type="text"
      placeholder="Search by ECO code or name..."
      bind:value={searchQuery}
      class="search-input"
      aria-label="Search openings by ECO code or name"
    />
  </div>

  {#if loading}
    <p class="loading">Loading...</p>
  {:else if error}
    <p class="error">{error}</p>
  {:else if filteredOpenings.length === 0}
    <p class="empty">No openings found</p>
  {:else}
    <table>
      <caption class="sr-only">Opening statistics showing ECO codes, names, and win percentages</caption>
      <thead>
        <tr>
          <th scope="col">ECO</th>
          <th scope="col">Name</th>
          <th scope="col">Games</th>
          <th scope="col">White %</th>
          <th scope="col">Draw %</th>
          <th scope="col">Black %</th>
        </tr>
      </thead>
      <tbody>
        {#each filteredOpenings as opening}
          <tr>
            <td class="eco">{opening.eco}</td>
            <td>{opening.name}</td>
            <td class="numeric">{opening.games_played}</td>
            <td class="numeric white-win">{percentage(opening.white_wins, opening.games_played)}</td>
            <td class="numeric draw">{percentage(opening.draws, opening.games_played)}</td>
            <td class="numeric black-win">{percentage(opening.black_wins, opening.games_played)}</td>
          </tr>
        {/each}
      </tbody>
    </table>

    <p class="stats">
      Showing {filteredOpenings.length} of {openings.length} openings
    </p>
  {/if}
</div>

<style>
  .openings-page {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  .search-container {
    width: 100%;
  }

  .search-input {
    width: 100%;
    max-width: 400px;
    padding: 0.75rem 1rem;
    background: var(--bg-secondary);
    border: 1px solid var(--accent);
    border-radius: 4px;
    color: var(--text);
    font-size: 1rem;
  }

  .search-input::placeholder {
    color: var(--text-muted);
  }

  .search-input:focus {
    outline: none;
    border-color: var(--highlight);
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
    color: var(--text-muted);
    font-weight: 500;
  }

  .eco {
    font-family: monospace;
    font-weight: bold;
    color: var(--highlight);
  }

  .numeric {
    text-align: right;
    font-family: monospace;
  }

  .white-win {
    color: var(--eval-white);
  }

  .draw {
    color: var(--text-muted);
  }

  .black-win {
    color: var(--warning);
  }

  tbody tr:hover {
    background: var(--accent);
  }

  .loading, .error, .empty {
    text-align: center;
    padding: 2rem;
    color: var(--text-muted);
  }

  .error {
    color: var(--highlight);
  }

  .stats {
    text-align: center;
    color: var(--text-muted);
    font-size: 0.875rem;
  }

  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border: 0;
  }
</style>
