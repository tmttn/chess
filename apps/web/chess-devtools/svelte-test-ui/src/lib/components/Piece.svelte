<script lang="ts">
  import { pieces, type PieceType } from '../pieces';
  import type { PieceInfo } from '../wasm';

  interface Props {
    piece: PieceInfo;
    draggable?: boolean;
  }

  let { piece, draggable = true }: Props = $props();

  const fillColor = $derived(piece.color === 'white' ? '#fff' : '#333');
  const strokeColor = $derived(piece.color === 'white' ? '#333' : '#fff');
</script>

<svg
  viewBox="0 0 45 45"
  class="piece"
  class:draggable
  role="img"
  aria-label={`${piece.color} ${piece.type}`}
>
  <g
    fill={fillColor}
    stroke={strokeColor}
    stroke-width="1.5"
    stroke-linecap="round"
    stroke-linejoin="round"
  >
    <path d={pieces[piece.type]} />
  </g>
</svg>

<style>
  .piece {
    width: 100%;
    height: 100%;
    pointer-events: none;
    user-select: none;
  }

  .draggable {
    cursor: grab;
    pointer-events: auto;
  }

  .draggable:active {
    cursor: grabbing;
  }
</style>
