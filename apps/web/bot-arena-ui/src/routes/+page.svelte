<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '$lib/api';
  import type { Bot, Match } from '$lib/types';

  let bots: Bot[] = $state([]);
  let recentMatches: Match[] = $state([]);
  let loading = $state(true);
  let error = $state<string | null>(null);

  onMount(async () => {
    try {
      [bots, recentMatches] = await Promise.all([
        api.getBots(),
        api.getMatches({ limit: 10 })
      ]);
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load data';
    } finally {
      loading = false;
    }
  });

  function winRate(bot: Bot): string {
    if (bot.games_played === 0) return '-';
    const rate = (bot.wins + bot.draws * 0.5) / bot.games_played * 100;
    return rate.toFixed(1) + '%';
  }
</script>

<div class="dashboard">
  <h1>Bot Arena Dashboard</h1>

  {#if loading}
    <p class="loading">Loading...</p>
  {:else if error}
    <p class="error">{error}</p>
  {:else}
    <section class="leaderboard">
      <h2>Leaderboard</h2>
      <table>
        <thead>
          <tr>
            <th>#</th>
            <th>Bot</th>
            <th>Elo</th>
            <th>W/L/D</th>
            <th>Win Rate</th>
          </tr>
        </thead>
        <tbody>
          {#each bots as bot, i}
            <tr>
              <td>{i + 1}</td>
              <td><a href="/bots/{bot.name}">{bot.name}</a></td>
              <td>{bot.elo_rating}</td>
              <td>{bot.wins}/{bot.losses}/{bot.draws}</td>
              <td>{winRate(bot)}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </section>

    <section class="recent-matches">
      <h2>Recent Matches</h2>
      {#if recentMatches.length === 0}
        <p class="empty">No matches yet</p>
      {:else}
        <ul>
          {#each recentMatches as match}
            <li>
              <a href="/games/{match.id}">
                {match.white_bot} vs {match.black_bot}
                ({match.white_score}-{match.black_score})
              </a>
              <span class="date">{new Date(match.started_at).toLocaleDateString()}</span>
            </li>
          {/each}
        </ul>
      {/if}
    </section>
  {/if}
</div>

<style>
  .dashboard {
    display: grid;
    gap: 2rem;
  }

  h1 {
    margin-bottom: 1rem;
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

  table {
    width: 100%;
    border-collapse: collapse;
  }

  th, td {
    padding: 0.75rem;
    text-align: left;
    border-bottom: 1px solid var(--accent);
  }

  th {
    color: var(--text-muted);
    font-weight: 500;
  }

  .recent-matches ul {
    list-style: none;
  }

  .recent-matches li {
    padding: 0.75rem 0;
    border-bottom: 1px solid var(--accent);
    display: flex;
    justify-content: space-between;
  }

  .date {
    color: var(--text-muted);
  }

  .loading, .error, .empty {
    text-align: center;
    padding: 2rem;
    color: var(--text-muted);
  }

  .error {
    color: var(--highlight);
  }
</style>
