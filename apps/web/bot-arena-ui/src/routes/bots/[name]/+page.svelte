<script lang="ts">
  import { browser } from '$app/environment';
  import { page } from '$app/stores';
  import { api, getExportUrl, type BotProfile, type EloHistoryPoint } from '$lib/api';

  let profile: BotProfile | null = $state(null);
  let loading = $state(true);
  let error = $state<string | null>(null);

  const name = $derived($page.params.name);

  const winRate = $derived.by(() => {
    if (!profile || profile.games_played === 0) return '-';
    const rate = ((profile.wins + profile.draws * 0.5) / profile.games_played) * 100;
    return rate.toFixed(1) + '%';
  });

  const recentHistory = $derived.by(() => {
    if (!profile) return [];
    return profile.elo_history.slice(-20);
  });

  $effect(() => {
    if (!browser) return;

    async function loadData() {
      if (!name) {
        error = 'No bot name provided';
        loading = false;
        return;
      }
      try {
        profile = await api.getBot(decodeURIComponent(name));
      } catch (e) {
        error = e instanceof Error ? e.message : 'Failed to load bot profile';
      } finally {
        loading = false;
      }
    }

    loadData();
  });

  /**
   * Format timestamp for display
   * @param timestamp - ISO timestamp string
   * @returns Formatted date string
   */
  function formatDate(timestamp: string): string {
    return new Date(timestamp).toLocaleDateString(undefined, {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  }

  /**
   * Format Elo change for display
   * @param current - Current Elo value
   * @param previous - Previous Elo value
   * @returns Formatted change string with sign
   */
  function formatEloChange(current: number, previous: number): string {
    const change = current - previous;
    if (change === 0) return '0';
    return change > 0 ? `+${change}` : `${change}`;
  }
</script>

<div class="bot-profile">
  {#if loading}
    <p class="loading">Loading...</p>
  {:else if error}
    <p class="error">{error}</p>
  {:else if profile}
    <header>
      <div class="header-row">
        <h1>{profile.name}</h1>
        {#if name}
          <a href={getExportUrl('bot', name)} download class="export-btn">
            Export Profile
          </a>
        {/if}
      </div>
      <p class="elo-rating">
        <span class="label">Elo Rating:</span>
        <span class="value">{profile.elo_rating}</span>
      </p>
    </header>

    <section class="stats" aria-label="Bot Statistics">
      <h2>Statistics</h2>
      <dl class="stats-grid">
        <div class="stat-item">
          <dt>Games Played</dt>
          <dd>{profile.games_played}</dd>
        </div>
        <div class="stat-item">
          <dt>Wins</dt>
          <dd class="wins">{profile.wins}</dd>
        </div>
        <div class="stat-item">
          <dt>Draws</dt>
          <dd class="draws">{profile.draws}</dd>
        </div>
        <div class="stat-item">
          <dt>Losses</dt>
          <dd class="losses">{profile.losses}</dd>
        </div>
        <div class="stat-item">
          <dt>Win Rate</dt>
          <dd>{winRate}</dd>
        </div>
      </dl>
    </section>

    <section class="elo-history" aria-label="Elo Rating History">
      <h2>Elo History</h2>
      {#if recentHistory.length === 0}
        <p class="empty">No rating history available</p>
      {:else}
        <div class="history-table-container">
          <table>
            <thead>
              <tr>
                <th scope="col">Date</th>
                <th scope="col">Elo</th>
                <th scope="col">Change</th>
              </tr>
            </thead>
            <tbody>
              {#each recentHistory as point, i}
                {@const previousElo = i > 0 ? recentHistory[i - 1].elo : point.elo}
                <tr>
                  <td>{formatDate(point.timestamp)}</td>
                  <td class="elo-value">{point.elo}</td>
                  <td class="elo-change" class:positive={point.elo > previousElo} class:negative={point.elo < previousElo}>
                    {i === 0 ? '-' : formatEloChange(point.elo, previousElo)}
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
        {#if profile.elo_history.length > 20}
          <p class="history-note">Showing last 20 of {profile.elo_history.length} entries</p>
        {/if}
      {/if}
    </section>

    <nav class="back-link">
      <a href="/">Back to Dashboard</a>
    </nav>
  {/if}
</div>

<style>
  .bot-profile {
    display: flex;
    flex-direction: column;
    gap: 2rem;
  }

  header {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .header-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 1rem;
  }

  h1 {
    font-size: 2rem;
    margin: 0;
  }

  .export-btn {
    display: inline-block;
    padding: 0.5rem 1rem;
    background: var(--accent);
    color: var(--text);
    text-decoration: none;
    border-radius: 4px;
    font-size: 0.9rem;
  }

  .export-btn:hover {
    background: var(--highlight);
  }

  .elo-rating {
    font-size: 1.25rem;
  }

  .elo-rating .label {
    color: var(--text-muted);
  }

  .elo-rating .value {
    font-weight: bold;
    color: var(--highlight);
    margin-left: 0.5rem;
  }

  h2 {
    margin-bottom: 1rem;
    color: var(--text-muted);
    font-size: 1.2rem;
  }

  section {
    background: var(--bg-secondary);
    padding: 1.5rem;
    border-radius: 8px;
  }

  /* Stats grid */
  .stats-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
    gap: 1rem;
  }

  .stat-item {
    background: var(--bg);
    padding: 1rem;
    border-radius: 6px;
    text-align: center;
  }

  .stat-item dt {
    color: var(--text-muted);
    font-size: 0.875rem;
    margin-bottom: 0.5rem;
  }

  .stat-item dd {
    font-size: 1.5rem;
    font-weight: bold;
  }

  .stat-item dd.wins {
    color: var(--success);
  }

  .stat-item dd.draws {
    color: var(--warning);
  }

  .stat-item dd.losses {
    color: var(--highlight);
  }

  /* Elo history table */
  .history-table-container {
    overflow-x: auto;
  }

  table {
    width: 100%;
    border-collapse: collapse;
  }

  th,
  td {
    padding: 0.75rem;
    text-align: left;
    border-bottom: 1px solid var(--accent);
  }

  th {
    color: var(--text-muted);
    font-weight: 500;
  }

  .elo-value {
    font-weight: bold;
  }

  .elo-change {
    font-family: monospace;
  }

  .elo-change.positive {
    color: var(--success);
  }

  .elo-change.negative {
    color: var(--highlight);
  }

  .history-note {
    margin-top: 1rem;
    color: var(--text-muted);
    font-size: 0.875rem;
    font-style: italic;
  }

  /* States */
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

  /* Navigation */
  .back-link {
    margin-top: 1rem;
  }

  @media (max-width: 600px) {
    .stats-grid {
      grid-template-columns: repeat(2, 1fr);
    }

    h1 {
      font-size: 1.5rem;
    }
  }
</style>
