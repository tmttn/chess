<script lang="ts">
  import { browser } from '$app/environment';
  import { api, type HeadToHeadMatrix, type HeadToHeadRecord } from '$lib/api';

  let matrix: HeadToHeadMatrix | null = $state(null);
  let loading = $state(true);
  let error = $state<string | null>(null);

  $effect(() => {
    if (!browser) return;

    async function loadData() {
      try {
        matrix = await api.getHeadToHead();
      } catch (e) {
        error = e instanceof Error ? e.message : 'Failed to load head-to-head data';
      } finally {
        loading = false;
      }
    }

    loadData();
  });

  /**
   * Find the record for a specific bot pairing
   * @param white - Bot playing as white
   * @param black - Bot playing as black
   * @returns The record if found, null otherwise
   */
  function getRecord(white: string, black: string): HeadToHeadRecord | null {
    if (!matrix) return null;
    return matrix.records.find((r) => r.white_bot === white && r.black_bot === black) ?? null;
  }

  /**
   * Format the head-to-head record between two bots
   * Combines results from both color directions
   * @param bot1 - First bot (row)
   * @param bot2 - Second bot (column)
   * @returns Formatted "W/D/L" string from bot1's perspective
   */
  function formatRecord(bot1: string, bot2: string): string {
    if (bot1 === bot2) return '-';

    // Get records for both color combinations
    const asWhite = getRecord(bot1, bot2);
    const asBlack = getRecord(bot2, bot1);

    // Calculate combined stats from bot1's perspective
    // When bot1 is white: white_wins are bot1's wins, black_wins are bot1's losses
    // When bot1 is black: black_wins are bot1's wins, white_wins are bot1's losses
    let wins = 0;
    let draws = 0;
    let losses = 0;

    if (asWhite) {
      wins += asWhite.white_wins;
      draws += asWhite.draws;
      losses += asWhite.black_wins;
    }

    if (asBlack) {
      wins += asBlack.black_wins;
      draws += asBlack.draws;
      losses += asBlack.white_wins;
    }

    const total = wins + draws + losses;
    if (total === 0) return '-';

    return `${wins}/${draws}/${losses}`;
  }

  /**
   * Get total games played between two bots
   * @param bot1 - First bot
   * @param bot2 - Second bot
   * @returns Total number of games
   */
  function getTotalGames(bot1: string, bot2: string): number {
    if (bot1 === bot2) return 0;

    const asWhite = getRecord(bot1, bot2);
    const asBlack = getRecord(bot2, bot1);

    return (asWhite?.games ?? 0) + (asBlack?.games ?? 0);
  }
</script>

<div class="stats-page">
  <h1>Head-to-Head Statistics</h1>

  {#if loading}
    <p class="loading">Loading...</p>
  {:else if error}
    <p class="error">{error}</p>
  {:else if !matrix || matrix.bots.length === 0}
    <p class="empty">No statistics available</p>
  {:else}
    <div class="matrix-container">
      <table class="matrix-table">
        <caption class="sr-only">
          Head-to-head results matrix showing wins, draws, and losses for each bot pairing
        </caption>
        <thead>
          <tr>
            <th class="corner" scope="col">
              <span class="sr-only">Bot name</span>
            </th>
            {#each matrix.bots as bot}
              <th class="column-header" scope="col">
                <span class="rotated">{bot}</span>
              </th>
            {/each}
          </tr>
        </thead>
        <tbody>
          {#each matrix.bots as rowBot}
            <tr>
              <th class="row-header" scope="row">{rowBot}</th>
              {#each matrix.bots as colBot}
                <td
                  class="cell"
                  class:diagonal={rowBot === colBot}
                  title={rowBot === colBot
                    ? ''
                    : `${rowBot} vs ${colBot}: ${getTotalGames(rowBot, colBot)} games`}
                >
                  {formatRecord(rowBot, colBot)}
                </td>
              {/each}
            </tr>
          {/each}
        </tbody>
      </table>
    </div>

    <div class="legend">
      <h2>Legend</h2>
      <p>
        Each cell shows <strong>W/D/L</strong> (Wins/Draws/Losses) from the perspective of the
        row bot against the column bot.
      </p>
      <p>Results are combined from games played as both white and black.</p>
    </div>
  {/if}
</div>

<style>
  .stats-page {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  .matrix-container {
    overflow-x: auto;
    background: var(--bg-secondary);
    border-radius: 8px;
    padding: 1rem;
  }

  .matrix-table {
    border-collapse: collapse;
    min-width: max-content;
  }

  .corner {
    width: 120px;
    min-width: 120px;
  }

  .column-header {
    height: 100px;
    vertical-align: bottom;
    padding: 0 0.25rem 0.5rem 0.25rem;
    text-align: left;
  }

  .rotated {
    display: inline-block;
    white-space: nowrap;
    transform: rotate(-90deg) translateX(-100%);
    transform-origin: left bottom;
    width: 1.5rem;
    font-weight: 500;
    color: var(--text-muted);
  }

  .row-header {
    text-align: left;
    padding: 0.5rem 1rem;
    font-weight: 500;
    color: var(--text-muted);
    white-space: nowrap;
  }

  .cell {
    text-align: center;
    padding: 0.5rem;
    font-family: monospace;
    font-size: 0.875rem;
    min-width: 80px;
    border: 1px solid var(--accent);
  }

  .diagonal {
    background: var(--accent);
    color: var(--text-muted);
  }

  tbody tr:hover .cell:not(.diagonal) {
    background: rgba(233, 69, 96, 0.1);
  }

  .legend {
    background: var(--bg-secondary);
    padding: 1rem 1.5rem;
    border-radius: 8px;
  }

  .legend h2 {
    font-size: 1rem;
    color: var(--text-muted);
    margin-bottom: 0.5rem;
  }

  .legend p {
    color: var(--text);
    font-size: 0.875rem;
    margin-bottom: 0.25rem;
  }

  .legend p:last-child {
    margin-bottom: 0;
  }

  .loading,
  .error,
  .empty {
    text-align: center;
    padding: 2rem;
    color: var(--text-muted);
  }

  .error {
    color: var(--highlight);
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
