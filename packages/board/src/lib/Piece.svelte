<script lang="ts">
  import type { PieceInfo } from './types';

  interface Props {
    piece: PieceInfo;
    draggable?: boolean;
  }

  let { piece, draggable = true }: Props = $props();

  const PIECE_SET = 'cburnett';
  const pieceMap: Record<string, string> = {
    k: 'K', q: 'Q', r: 'R', b: 'B', n: 'N', p: 'P'
  };

  const pieceName = $derived(() => {
    const colorPrefix = piece.color === 'white' ? 'w' : 'b';
    return `${colorPrefix}${pieceMap[piece.type]}`;
  });

  const src = $derived(`https://lichess1.org/assets/piece/${PIECE_SET}/${pieceName()}.svg`);
</script>

<img {src} alt={`${piece.color} ${piece.type}`} class="piece" class:draggable draggable="false" />

<style>
  .piece { width: 100%; height: 100%; pointer-events: none; user-select: none; -webkit-user-drag: none; }
  .draggable { cursor: grab; pointer-events: auto; }
  .draggable:active { cursor: grabbing; }
</style>
