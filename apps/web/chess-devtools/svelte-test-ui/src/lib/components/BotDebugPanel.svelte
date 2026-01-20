<script lang="ts">
  import { botStore, searchInfo, lastOutput, isConnected } from '$lib/stores/bot';
  import { isViewingHistory, viewSearchInfo } from '$lib/stores/game';

  let showRawOutput = $state(false);
  let connected = $derived($isConnected);
  let liveInfo = $derived($searchInfo);
  let historyInfo = $derived($viewSearchInfo);
  let viewingHistory = $derived($isViewingHistory);
  let output = $derived($lastOutput);

  // Show stored info from viewed move if available, otherwise live info (during active search)
  let info = $derived(historyInfo ?? liveInfo);

  function formatScore(score: number): string {
    if (Math.abs(score) >= 90000) {
      const mateIn = score > 0 ? 100000 - score : -100000 - score;
      return `M${mateIn > 0 ? mateIn : -mateIn}`;
    }
    return (score / 100).toFixed(2);
  }

  function formatNodes(nodes: number): string {
    if (nodes >= 1_000_000) {
      return `${(nodes / 1_000_000).toFixed(1)}M`;
    }
    if (nodes >= 1_000) {
      return `${(nodes / 1_000).toFixed(1)}k`;
    }
    return nodes.toString();
  }

  function formatNps(nodes: number, time: number): string {
    if (time === 0) return '-';
    const nps = (nodes / time) * 1000;
    return formatNodes(Math.round(nps)) + '/s';
  }
</script>

<div class="debug-panel">
  <div class="header">
    <h3>Bot Debug</h3>
    <label class="toggle">
      <input type="checkbox" bind:checked={showRawOutput} />
      Raw
    </label>
  </div>

  {#if viewingHistory && historyInfo}
    <div class="history-notice">Viewing move's search info</div>
  {/if}

  {#if !connected && !viewingHistory}
    <div class="placeholder">
      Connect to a bot to see debug info
    </div>
  {:else if showRawOutput && !viewingHistory}
    <div class="raw-output">
      {#each output.slice(-20) as line}
        <div class="output-line">{line}</div>
      {/each}
    </div>
  {:else if info}
    <div class="search-info">
      <div class="info-grid">
        <div class="info-item">
          <span class="label">Depth</span>
          <span class="value">{info.depth}</span>
        </div>
        <div class="info-item">
          <span class="label">Score</span>
          <span class="value" class:positive={info.score > 0} class:negative={info.score < 0}>
            {info.score > 0 ? '+' : ''}{formatScore(info.score)}
          </span>
        </div>
        <div class="info-item">
          <span class="label">Nodes</span>
          <span class="value">{formatNodes(info.nodes)}</span>
        </div>
        <div class="info-item">
          <span class="label">Time</span>
          <span class="value">{info.time}ms</span>
        </div>
        <div class="info-item">
          <span class="label">NPS</span>
          <span class="value">{formatNps(info.nodes, info.time)}</span>
        </div>
      </div>

      {#if info.pv.length > 0}
        <div class="pv-section">
          <span class="label">PV</span>
          <span class="pv-moves">{info.pv.join(' ')}</span>
        </div>
      {/if}
    </div>
  {:else if viewingHistory}
    <div class="placeholder">
      No search info for this move
    </div>
  {:else}
    <div class="placeholder">
      Waiting for search info...
    </div>
  {/if}
</div>

<style>
  .debug-panel {
    background: var(--bg-secondary);
    border-radius: 8px;
    padding: 1rem;
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.75rem;
  }

  h3 {
    margin: 0;
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  .toggle {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    font-size: 0.75rem;
    color: var(--text-muted);
    cursor: pointer;
  }

  .toggle input {
    cursor: pointer;
  }

  .history-notice {
    background: var(--accent);
    color: white;
    padding: 0.375rem 0.5rem;
    border-radius: 4px;
    font-size: 0.7rem;
    margin-bottom: 0.75rem;
    text-align: center;
  }

  .placeholder {
    color: var(--text-muted);
    font-size: 0.8rem;
    font-style: italic;
    text-align: center;
    padding: 1rem;
  }

  .search-info {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .info-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 0.5rem;
  }

  .info-item {
    display: flex;
    flex-direction: column;
    gap: 0.125rem;
  }

  .label {
    font-size: 0.7rem;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .value {
    font-size: 0.9rem;
    font-weight: 500;
    font-family: monospace;
  }

  .value.positive {
    color: #4caf50;
  }

  .value.negative {
    color: #f44336;
  }

  .pv-section {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    padding-top: 0.5rem;
    border-top: 1px solid var(--bg-tertiary);
  }

  .pv-moves {
    font-family: monospace;
    font-size: 0.8rem;
    word-break: break-all;
    color: var(--text-secondary);
  }

  .raw-output {
    max-height: 200px;
    overflow-y: auto;
    font-family: monospace;
    font-size: 0.7rem;
    background: var(--bg-primary);
    border-radius: 4px;
    padding: 0.5rem;
  }

  .output-line {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    color: var(--text-muted);
    padding: 0.125rem 0;
  }

  .output-line:nth-child(odd) {
    background: var(--bg-secondary);
  }
</style>
