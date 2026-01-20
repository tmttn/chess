import { writable } from 'svelte/store';
import { browser } from '$app/environment';

// Lichess sound set - standard theme
const SOUND_BASE = 'https://lichess1.org/assets/sound/standard';

type SoundType = 'move' | 'capture' | 'check' | 'gameStart' | 'gameEnd';

const soundUrls: Record<SoundType, string> = {
  move: `${SOUND_BASE}/Move.mp3`,
  capture: `${SOUND_BASE}/Capture.mp3`,
  check: `${SOUND_BASE}/Check.mp3`,
  gameStart: `${SOUND_BASE}/GenericNotify.mp3`,
  gameEnd: `${SOUND_BASE}/Victory.mp3`,
};

// Mute state with localStorage persistence
const MUTE_KEY = 'chess-devtools-muted';
const initialMuted = browser ? localStorage.getItem(MUTE_KEY) === 'true' : false;
export const isMuted = writable(initialMuted);

// Persist mute state
isMuted.subscribe((muted) => {
  if (browser) {
    localStorage.setItem(MUTE_KEY, String(muted));
  }
});

export function toggleMute(): void {
  isMuted.update((m) => !m);
}

// Audio cache for instant playback
const audioCache = new Map<SoundType, HTMLAudioElement>();
let currentMuted = initialMuted;

// Keep track of mute state
isMuted.subscribe((m) => {
  currentMuted = m;
});

function getAudio(type: SoundType): HTMLAudioElement {
  let audio = audioCache.get(type);
  if (!audio) {
    audio = new Audio(soundUrls[type]);
    audio.volume = 0.5;
    audioCache.set(type, audio);
  }
  return audio;
}

export function playSound(type: SoundType): void {
  if (currentMuted) return;
  const audio = getAudio(type);
  audio.currentTime = 0;
  audio.play().catch(() => {
    // Ignore autoplay restrictions - user needs to interact first
  });
}

// Preload sounds for instant playback
export function preloadSounds(): void {
  Object.keys(soundUrls).forEach((type) => {
    getAudio(type as SoundType);
  });
}
