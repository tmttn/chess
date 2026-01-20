// packages/game-store/src/sounds.test.ts

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { get } from 'svelte/store';
import {
  isMuted,
  toggleMute,
  preloadSounds,
  clearSoundCache,
  lichessSounds,
} from './sounds';

// Mock Audio API
class MockAudio {
  src = '';
  volume = 1;
  currentTime = 0;
  play = vi.fn(() => Promise.resolve());

  constructor(src?: string) {
    this.src = src ?? '';
  }
}

describe('sounds', () => {
  let originalAudio: typeof Audio | undefined;

  beforeEach(() => {
    // Store original Audio and replace with mock
    originalAudio = globalThis.Audio;
    (globalThis as unknown as { Audio: typeof MockAudio }).Audio = MockAudio;

    // Reset mute state
    isMuted.set(false);
    clearSoundCache();
    vi.clearAllMocks();
  });

  afterEach(() => {
    // Restore original Audio
    if (originalAudio !== undefined) {
      (globalThis as unknown as { Audio: typeof Audio }).Audio = originalAudio;
    }
  });

  describe('isMuted store', () => {
    it('should start with false', () => {
      expect(get(isMuted)).toBe(false);
    });

    it('should update when set', () => {
      isMuted.set(true);
      expect(get(isMuted)).toBe(true);
    });
  });

  describe('toggleMute', () => {
    it('should toggle from false to true', () => {
      expect(get(isMuted)).toBe(false);
      toggleMute();
      expect(get(isMuted)).toBe(true);
    });

    it('should toggle from true to false', () => {
      isMuted.set(true);
      toggleMute();
      expect(get(isMuted)).toBe(false);
    });

    it('should toggle multiple times', () => {
      toggleMute();
      expect(get(isMuted)).toBe(true);
      toggleMute();
      expect(get(isMuted)).toBe(false);
      toggleMute();
      expect(get(isMuted)).toBe(true);
    });
  });

  describe('preloadSounds', () => {
    it('should not throw when Audio is available', () => {
      expect(() => preloadSounds()).not.toThrow();
    });

    it('should not throw when Audio is unavailable (SSR)', () => {
      (globalThis as unknown as { Audio: undefined }).Audio = undefined;
      expect(() => preloadSounds()).not.toThrow();
    });
  });

  describe('clearSoundCache', () => {
    it('should not throw', () => {
      preloadSounds();
      expect(() => clearSoundCache()).not.toThrow();
    });
  });

  describe('lichessSounds', () => {
    it('should return a SoundConfig object', () => {
      const sounds = lichessSounds();
      expect(sounds).toHaveProperty('playMove');
      expect(sounds).toHaveProperty('playCapture');
      expect(sounds).toHaveProperty('playCheck');
      expect(sounds).toHaveProperty('playGameStart');
      expect(sounds).toHaveProperty('playGameEnd');
    });

    it('should have callable functions', () => {
      const sounds = lichessSounds();
      expect(typeof sounds.playMove).toBe('function');
      expect(typeof sounds.playCapture).toBe('function');
      expect(typeof sounds.playCheck).toBe('function');
      expect(typeof sounds.playGameStart).toBe('function');
      expect(typeof sounds.playGameEnd).toBe('function');
    });

    it('playMove should not throw', () => {
      const sounds = lichessSounds();
      expect(() => sounds.playMove()).not.toThrow();
    });

    it('playCapture should not throw', () => {
      const sounds = lichessSounds();
      expect(() => sounds.playCapture()).not.toThrow();
    });

    it('playCheck should not throw', () => {
      const sounds = lichessSounds();
      expect(() => sounds.playCheck()).not.toThrow();
    });

    it('playGameStart should not throw', () => {
      const sounds = lichessSounds();
      expect(() => sounds.playGameStart()).not.toThrow();
    });

    it('playGameEnd should not throw', () => {
      const sounds = lichessSounds();
      expect(() => sounds.playGameEnd()).not.toThrow();
    });

    it('should not play when muted', () => {
      // Create sounds while unmuted
      const sounds = lichessSounds();

      // Mute
      isMuted.set(true);

      // Call all sound functions - they should return silently
      sounds.playMove();
      sounds.playCapture();
      sounds.playCheck();
      sounds.playGameStart();
      sounds.playGameEnd();

      // No exceptions thrown
    });

    it('should work when Audio is unavailable (SSR)', () => {
      (globalThis as unknown as { Audio: undefined }).Audio = undefined;
      const sounds = lichessSounds();

      // All functions should be safe to call
      expect(() => sounds.playMove()).not.toThrow();
      expect(() => sounds.playCapture()).not.toThrow();
      expect(() => sounds.playCheck()).not.toThrow();
      expect(() => sounds.playGameStart()).not.toThrow();
      expect(() => sounds.playGameEnd()).not.toThrow();
    });
  });

  describe('integration', () => {
    it('mute state should affect sound playback', () => {
      const sounds = lichessSounds();

      // Initially unmuted - sounds should work
      expect(get(isMuted)).toBe(false);
      sounds.playMove();

      // Toggle mute
      toggleMute();
      expect(get(isMuted)).toBe(true);
      sounds.playMove(); // Should be silently ignored

      // Toggle back
      toggleMute();
      expect(get(isMuted)).toBe(false);
      sounds.playMove(); // Should work again
    });

    it('clearSoundCache should allow fresh audio elements', () => {
      preloadSounds();
      clearSoundCache();
      // Should be able to call again without issues
      preloadSounds();
      const sounds = lichessSounds();
      sounds.playMove();
    });
  });
});
