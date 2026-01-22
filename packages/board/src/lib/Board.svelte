<script lang="ts">
  import Square from './Square.svelte';
  import PromotionDialog from './PromotionDialog.svelte';
  import type { Move, PieceInfo, LastMove } from './types';

  interface Props {
    board: Map<string, PieceInfo>;
    legalMoves: Move[];
    flipped?: boolean;
    lastMove?: LastMove | null;
    check?: string | null;
    sideToMove?: 'white' | 'black';
    onMove?: (from: string, to: string, promotion?: string) => void;
  }

  let {
    board,
    legalMoves,
    flipped = false,
    lastMove = null,
    check = null,
    sideToMove = 'white',
    onMove
  }: Props = $props();

  let selectedSquare: string | null = $state(null);
  let pendingPromotion: { from: string; to: string } | null = $state(null);

  const files = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];
  const ranks = ['8', '7', '6', '5', '4', '3', '2', '1'];

  const displayFiles = $derived(flipped ? [...files].reverse() : files);
  const displayRanks = $derived(flipped ? [...ranks].reverse() : ranks);

  function isLightSquare(file: string, rank: string): boolean {
    const fileIdx = files.indexOf(file);
    const rankIdx = parseInt(rank);
    return (fileIdx + rankIdx) % 2 === 1;
  }

  function getLegalMovesFrom(square: string): Move[] {
    return legalMoves.filter(m => m.from === square);
  }

  function isLegalTarget(square: string): boolean {
    if (!selectedSquare) return false;
    return getLegalMovesFrom(selectedSquare).some(m => m.to === square);
  }

  function isKingInCheck(square: string): boolean {
    return check === square;
  }

  function isPromotionMove(from: string, to: string): boolean {
    const moves = getLegalMovesFrom(from).filter(m => m.to === to);
    return moves.some(m => m.promotion);
  }

  function executeMove(from: string, to: string, promotion?: string) {
    if (onMove) {
      onMove(from, to, promotion);
    }
    selectedSquare = null;
    pendingPromotion = null;
  }

  function handlePromotionSelect(piece: 'q' | 'r' | 'b' | 'n') {
    if (pendingPromotion) {
      executeMove(pendingPromotion.from, pendingPromotion.to, piece);
    }
  }

  function handlePromotionCancel() {
    pendingPromotion = null;
    selectedSquare = null;
  }

  function handleSquareClick(square: string) {
    const piece = board.get(square);

    // If we have a selected piece and clicking a legal target
    if (selectedSquare && isLegalTarget(square)) {
      if (isPromotionMove(selectedSquare, square)) {
        pendingPromotion = { from: selectedSquare, to: square };
      } else {
        executeMove(selectedSquare, square);
      }
      return;
    }

    // If clicking own piece, select it
    if (piece && piece.color === sideToMove) {
      selectedSquare = square;
      return;
    }

    // Otherwise deselect
    selectedSquare = null;
  }

  function handleDragStart(e: DragEvent, square: string) {
    const piece = board.get(square);
    if (!piece || piece.color !== sideToMove || !e.dataTransfer) {
      e.preventDefault();
      return;
    }
    selectedSquare = square;
    e.dataTransfer.effectAllowed = 'move';
    e.dataTransfer.setData('text/plain', square);

    // Create custom drag image (just the piece, no yellow background)
    const img = (e.target as HTMLElement).querySelector('img');
    if (img) {
      const size = img.offsetWidth;
      e.dataTransfer.setDragImage(img, size / 2, size / 2);
    }
  }

  function handleDragOver(e: DragEvent, square: string) {
    if (selectedSquare && isLegalTarget(square) && e.dataTransfer) {
      e.preventDefault();
      e.dataTransfer.dropEffect = 'move';
    }
  }

  function handleDrop(e: DragEvent, square: string) {
    e.preventDefault();
    if (selectedSquare && isLegalTarget(square)) {
      if (isPromotionMove(selectedSquare, square)) {
        pendingPromotion = { from: selectedSquare, to: square };
      } else {
        executeMove(selectedSquare, square);
      }
    } else {
      selectedSquare = null;
    }
  }

  // Reset selection when board changes
  $effect(() => {
    board; // Subscribe to board changes
    selectedSquare = null;
  });
</script>

<div class="board-container">
  <div class="board">
    {#each displayRanks as rank}
      {#each displayFiles as file}
        {@const square = file + rank}
        {@const piece = board.get(square) ?? null}
        <Square
          {square}
          {piece}
          isLight={isLightSquare(file, rank)}
          isSelected={selectedSquare === square}
          isLegalTarget={isLegalTarget(square)}
          isLastMove={lastMove?.from === square || lastMove?.to === square}
          isCheck={isKingInCheck(square)}
          onclick={() => handleSquareClick(square)}
          ondragstart={(e) => handleDragStart(e, square)}
          ondragover={(e) => handleDragOver(e, square)}
          ondrop={(e) => handleDrop(e, square)}
        />
      {/each}
    {/each}
  </div>
  <div class="coordinates files">
    {#each displayFiles as file}
      <span>{file}</span>
    {/each}
  </div>
  <div class="coordinates ranks">
    {#each displayRanks as rank}
      <span>{rank}</span>
    {/each}
  </div>
</div>

{#if pendingPromotion}
  <PromotionDialog
    color={sideToMove}
    onselect={handlePromotionSelect}
    oncancel={handlePromotionCancel}
  />
{/if}

<style>
  .board-container {
    position: relative;
    display: inline-block;

    /* Default CSS variables - can be overridden by parent */
    --square-size: 60px;
    --square-light: #f0d9b5;
    --square-dark: #b58863;
    --square-selected: #829769;
    --square-last-move: rgba(155, 199, 0, 0.4);
    --square-legal: rgba(0, 0, 0, 0.2);
    --square-check: #e94560;
    --bg-tertiary: #374151;
    --text-muted: #9ca3af;
    --radius: 4px;
  }

  .board {
    display: grid;
    grid-template-columns: repeat(8, var(--square-size));
    grid-template-rows: repeat(8, var(--square-size));
    border: 2px solid var(--bg-tertiary);
    border-radius: var(--radius);
    overflow: hidden;
  }

  .coordinates {
    position: absolute;
    display: grid;
    color: var(--text-muted);
    font-size: 0.75rem;
    font-weight: 500;
  }

  .coordinates span {
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .coordinates.files {
    bottom: -1.5rem;
    left: 0;
    grid-template-columns: repeat(8, var(--square-size));
  }

  .coordinates.ranks {
    top: 0;
    left: -1.25rem;
    grid-template-rows: repeat(8, var(--square-size));
  }
</style>
