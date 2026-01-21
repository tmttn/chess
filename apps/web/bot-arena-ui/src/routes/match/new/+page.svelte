<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { api } from '$lib/api';
  import type { Bot } from '$lib/types';

  let bots: Bot[] = $state([]);
  let whiteBot = $state('');
  let blackBot = $state('');
  let games = $state(10);
  let movetime = $state(1000);
  let submitting = $state(false);
  let error = $state<string | null>(null);
  let loading = $state(true);

  onMount(async () => {
    try {
      bots = await api.getBots();
      if (bots.length >= 2) {
        whiteBot = bots[0].name;
        blackBot = bots[1].name;
      }
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load bots';
    } finally {
      loading = false;
    }
  });

  async function handleSubmit(e: Event) {
    e.preventDefault();
    if (whiteBot === blackBot) {
      error = 'Please select different bots';
      return;
    }

    submitting = true;
    error = null;

    try {
      const match = await api.createMatch({
        white_bot: whiteBot,
        black_bot: blackBot,
        games,
        movetime_ms: movetime,
      });
      goto(`/match/live/${match.id}`);
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to create match';
      submitting = false;
    }
  }
</script>

<div class="new-match">
  <h1>New Match</h1>

  {#if loading}
    <p class="loading">Loading bots...</p>
  {:else if bots.length < 2}
    <p class="error">At least 2 bots are required to create a match.</p>
  {:else}
    <form onsubmit={handleSubmit}>
      <div class="field">
        <label for="white-bot">White Bot</label>
        <select id="white-bot" bind:value={whiteBot} disabled={submitting}>
          {#each bots as bot}
            <option value={bot.name}>{bot.name} ({bot.elo_rating})</option>
          {/each}
        </select>
      </div>

      <div class="field">
        <label for="black-bot">Black Bot</label>
        <select id="black-bot" bind:value={blackBot} disabled={submitting}>
          {#each bots as bot}
            <option value={bot.name}>{bot.name} ({bot.elo_rating})</option>
          {/each}
        </select>
      </div>

      <div class="field">
        <label for="games">Number of Games</label>
        <input
          type="number"
          id="games"
          bind:value={games}
          min="1"
          max="100"
          disabled={submitting}
        />
      </div>

      <div class="field">
        <label for="movetime">Move Time (ms)</label>
        <input
          type="number"
          id="movetime"
          bind:value={movetime}
          min="100"
          max="60000"
          step="100"
          disabled={submitting}
        />
      </div>

      {#if error}
        <p class="error">{error}</p>
      {/if}

      <button type="submit" disabled={submitting}>
        {submitting ? 'Creating...' : 'Start Match'}
      </button>
    </form>
  {/if}
</div>

<style>
  .new-match {
    max-width: 400px;
    margin: 0 auto;
  }

  h1 {
    font-size: 1.5rem;
    margin-bottom: 1.5rem;
  }

  form {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  label {
    font-size: 0.875rem;
    color: var(--text-muted);
  }

  select,
  input {
    padding: 0.75rem;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--bg-secondary);
    color: var(--text);
    font-size: 1rem;
  }

  select:focus,
  input:focus {
    outline: none;
    border-color: var(--highlight);
  }

  select:disabled,
  input:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  button {
    margin-top: 1rem;
    padding: 0.75rem 1.5rem;
    border: none;
    border-radius: 8px;
    background: var(--highlight);
    color: var(--bg);
    font-size: 1rem;
    font-weight: 500;
    cursor: pointer;
    transition: opacity 0.2s;
  }

  button:hover:not(:disabled) {
    opacity: 0.9;
  }

  button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .loading {
    text-align: center;
    color: var(--text-muted);
  }

  .error {
    color: var(--highlight);
    font-size: 0.875rem;
  }
</style>
