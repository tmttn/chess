<script lang="ts">
  import Piece from './Piece.svelte';
  import type { PieceInfo } from './types';

  interface Props {
    square: string;
    piece: PieceInfo | null;
    isLight: boolean;
    isSelected: boolean;
    isLegalTarget: boolean;
    isLastMove: boolean;
    isCheck: boolean;
    onclick: () => void;
    ondragstart: (e: DragEvent) => void;
    ondragover: (e: DragEvent) => void;
    ondrop: (e: DragEvent) => void;
  }

  let { square, piece, isLight, isSelected, isLegalTarget, isLastMove, isCheck, onclick, ondragstart, ondragover, ondrop }: Props = $props();
</script>

<div class="square" class:light={isLight} class:dark={!isLight} class:selected={isSelected} class:legal-target={isLegalTarget} class:last-move={isLastMove} class:check={isCheck} data-square={square} role="button" tabindex="0" {onclick} onkeydown={(e) => e.key === 'Enter' && onclick()} {ondragover} {ondrop}>
  {#if piece}
    <div class="piece-wrapper" draggable="true" role="button" tabindex="0" {ondragstart}>
      <Piece {piece} />
    </div>
  {/if}
  {#if isLegalTarget && !piece}
    <div class="legal-dot"></div>
  {/if}
  {#if isLegalTarget && piece}
    <div class="legal-ring"></div>
  {/if}
</div>

<style>
  .square { position: relative; width: var(--square-size); height: var(--square-size); display: flex; align-items: center; justify-content: center; }
  .light { background: var(--square-light); }
  .dark { background: var(--square-dark); }
  .selected { background: var(--square-selected) !important; }
  .last-move { background: var(--square-last-move); }
  .light.last-move { background: color-mix(in srgb, var(--square-light), var(--square-last-move)); }
  .dark.last-move { background: color-mix(in srgb, var(--square-dark), var(--square-last-move)); }
  .check { background: var(--square-check) !important; }
  .piece-wrapper { width: 85%; height: 85%; cursor: grab; }
  .piece-wrapper:active { cursor: grabbing; }
  .legal-dot { width: 30%; height: 30%; border-radius: 50%; background: var(--square-legal); }
  .legal-ring { position: absolute; inset: 0; border: 4px solid var(--square-legal); border-radius: 50%; pointer-events: none; }
</style>
