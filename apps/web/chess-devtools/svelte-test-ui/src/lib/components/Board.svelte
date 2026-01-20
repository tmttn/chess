<script lang="ts">
  import Square from './Square.svelte';
  import { gameStore, board, legalMoves, isCheck, sideToMove } from '../stores/game';
  import type { Move, PieceInfo } from '../wasm';

  interface Props {
    flipped?: boolean;
  }

  let { flipped = false }: Props = $props();

  let selectedSquare: string | null = $state(null);
  let lastMove: { from: string; to: string } | null = $state(null);

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
    return $legalMoves.filter(m => m.from === square);
  }

  function isLegalTarget(square: string): boolean {
    if (!selectedSquare) return false;
    return getLegalMovesFrom(selectedSquare).some(m => m.to === square);
  }

  function isKingInCheck(square: string): boolean {
    if (!$isCheck) return false;
    const piece = $board.get(square);
    return piece?.type === 'k' && piece.color === $sideToMove;
  }

  function handleSquareClick(square: string) {
    const piece = $board.get(square);

    // If we have a selected piece and clicking a legal target
    if (selectedSquare && isLegalTarget(square)) {
      const move = getLegalMovesFrom(selectedSquare).find(m => m.to === square);
      if (move) {
        // Handle promotion - for now just promote to queen
        const uci = move.promotion ? move.uci : move.uci;
        gameStore.makeMove(uci);
        lastMove = { from: selectedSquare, to: square };
        selectedSquare = null;
      }
      return;
    }

    // If clicking own piece, select it
    if (piece && piece.color === $sideToMove) {
      selectedSquare = square;
      return;
    }

    // Otherwise deselect
    selectedSquare = null;
  }

  function handleDragStart(e: DragEvent, square: string) {
    const piece = $board.get(square);
    if (!piece || piece.color !== $sideToMove) {
      e.preventDefault();
      return;
    }
    selectedSquare = square;
    e.dataTransfer!.effectAllowed = 'move';
    e.dataTransfer!.setData('text/plain', square);
  }

  function handleDragOver(e: DragEvent, square: string) {
    if (selectedSquare && isLegalTarget(square)) {
      e.preventDefault();
      e.dataTransfer!.dropEffect = 'move';
    }
  }

  function handleDrop(e: DragEvent, square: string) {
    e.preventDefault();
    if (selectedSquare && isLegalTarget(square)) {
      const move = getLegalMovesFrom(selectedSquare).find(m => m.to === square);
      if (move) {
        gameStore.makeMove(move.uci);
        lastMove = { from: selectedSquare, to: square };
      }
    }
    selectedSquare = null;
  }

  // Reset selection when game changes
  $effect(() => {
    $board; // Subscribe to board changes
    selectedSquare = null;
  });
</script>

<div class="board-container">
  <div class="board">
    {#each displayRanks as rank}
      {#each displayFiles as file}
        {@const square = file + rank}
        {@const piece = $board.get(square) ?? null}
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

<style>
  .board-container {
    position: relative;
    display: inline-block;
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
    display: flex;
    color: var(--text-muted);
    font-size: 0.75rem;
    font-weight: 500;
  }

  .coordinates.files {
    bottom: -1.5rem;
    left: 0;
    right: 0;
    justify-content: space-around;
    padding: 0 calc(var(--square-size) * 0.3);
  }

  .coordinates.ranks {
    top: 0;
    bottom: 0;
    left: -1.25rem;
    flex-direction: column;
    justify-content: space-around;
    align-items: center;
    padding: calc(var(--square-size) * 0.3) 0;
  }
</style>
