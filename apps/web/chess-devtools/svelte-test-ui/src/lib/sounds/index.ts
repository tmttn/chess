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

// Audio cache for instant playback
const audioCache = new Map<SoundType, HTMLAudioElement>();

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
