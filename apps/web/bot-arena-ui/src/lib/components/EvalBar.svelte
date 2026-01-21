<script lang="ts">
  interface Props {
    scoreCp: number | null;
    scoreMate: number | null;
  }

  let { scoreCp, scoreMate }: Props = $props();

  // Convert score to percentage (0-100, 50 = equal)
  const percentage = $derived.by(() => {
    if (scoreMate !== null) {
      return scoreMate > 0 ? 100 : 0;
    }
    if (scoreCp === null) return 50;

    // Sigmoid-like scaling: ±1000cp maps to 0-100%
    const clamped = Math.max(-1000, Math.min(1000, scoreCp));
    return 50 + (clamped / 20);
  });

  const whiteHeight = $derived(`${percentage}%`);
  const blackHeight = $derived(`${100 - percentage}%`);

  // Format score for accessibility
  const scoreLabel = $derived.by(() => {
    if (scoreMate !== null) {
      return scoreMate > 0 ? `White mates in ${scoreMate}` : `Black mates in ${Math.abs(scoreMate)}`;
    }
    if (scoreCp !== null) {
      const pawns = (scoreCp / 100).toFixed(1);
      return scoreCp >= 0 ? `White +${pawns}` : `Black +${Math.abs(scoreCp / 100).toFixed(1)}`;
    }
    return 'Equal position';
  });
</script>

<div
  class="eval-bar"
  role="meter"
  aria-label="Position evaluation"
  aria-valuemin={0}
  aria-valuemax={100}
  aria-valuenow={percentage}
  title={scoreLabel}
>
  <div class="black-side" style="height: {blackHeight}"></div>
  <div class="white-side" style="height: {whiteHeight}"></div>
</div>

<style>
  .eval-bar {
    width: 20px;
    height: 100%;
    display: flex;
    flex-direction: column;
    border: 1px solid var(--border);
    border-radius: 4px;
    overflow: hidden;
  }

  .black-side {
    background: var(--eval-black);
    transition: height 0.3s ease;
  }

  .white-side {
    background: var(--eval-white);
    transition: height 0.3s ease;
  }
</style>
