// packages/game-store/src/sounds.ts

import { writable } from 'svelte/store';
import type { SoundConfig } from './types';

const SOUND_BASE = 'https://lichess1.org/assets/sound/standard';

type SoundType = 'move' | 'capture' | 'check' | 'gameStart' | 'gameEnd';

const soundUrls: Record<SoundType, string> = {
  move: `${SOUND_BASE}/Move.mp3`,
  capture: `${SOUND_BASE}/Capture.mp3`,
  check: `${SOUND_BASE}/Check.mp3`,
  gameStart: `${SOUND_BASE}/GenericNotify.mp3`,
  gameEnd: `${SOUND_BASE}/Victory.mp3`,
};

/** Mute state store - persisted to localStorage if available */
export const isMuted = writable(false);

let currentMuted = false;
isMuted.subscribe((m) => (currentMuted = m));

/** Toggle mute state */
export function toggleMute(): void {
  isMuted.update((m) => !m);
}

const audioCache = new Map<SoundType, HTMLAudioElement>();

function getAudio(type: SoundType): HTMLAudioElement | null {
  if (typeof Audio === 'undefined') return null;

  let audio = audioCache.get(type);
  if (!audio) {
    audio = new Audio(soundUrls[type]);
    audio.volume = 0.5;
    audioCache.set(type, audio);
  }
  return audio;
}

function playSound(type: SoundType): void {
  if (currentMuted) return;
  const audio = getAudio(type);
  if (audio) {
    audio.currentTime = 0;
    audio.play().catch(() => {});
  }
}

/** Preload all sounds for instant playback */
export function preloadSounds(): void {
  Object.keys(soundUrls).forEach((type) => {
    getAudio(type as SoundType);
  });
}

/**
 * Creates a Lichess-style sound configuration.
 * Call preloadSounds() on init for best experience.
 */
export function lichessSounds(): SoundConfig {
  return {
    playMove: () => playSound('move'),
    playCapture: () => playSound('capture'),
    playCheck: () => playSound('check'),
    playGameStart: () => playSound('gameStart'),
    playGameEnd: () => playSound('gameEnd'),
  };
}
