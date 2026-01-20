<script lang="ts">
  import '../app.css';
  import { browser } from '$app/environment';
  import { initWasm, wasmReady } from '$lib/wasm';
  import { gameStore } from '$lib/stores/game';

  let error: string | null = $state(null);
  let initStarted = false;

  $effect(() => {
    if (browser && !initStarted && !$wasmReady) {
      initStarted = true;
      console.log('Starting WASM initialization...');

      initWasm()
        .then(() => {
          console.log('WASM initialized successfully');
          gameStore.init();
          console.log('Game store initialized');
        })
        .catch((e) => {
          console.error('WASM init error:', e);
          error = e instanceof Error ? e.message : String(e);
        });
    }
  });
</script>

<svelte:head>
  <title>Chess Devtools - Test UI</title>
</svelte:head>

{#if error}
  <div class="error">
    <h1>Error loading chess engine</h1>
    <p>{error}</p>
  </div>
{:else if $wasmReady}
  <slot />
{:else}
  <div class="loading">
    <p>Loading chess engine...</p>
  </div>
{/if}

<style>
  .loading, .error {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    min-height: 100vh;
    color: var(--text-secondary);
  }

  .error {
    color: var(--error);
  }

  .error p {
    margin-top: 1rem;
    font-family: monospace;
  }
</style>
