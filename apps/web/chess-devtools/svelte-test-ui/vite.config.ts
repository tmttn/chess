import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [sveltekit()],
  optimizeDeps: {
    exclude: ['chess-wasm']
  },
  server: {
    fs: {
      // Allow serving files from the parent directories (for WASM)
      allow: ['../../../..']
    }
  },
  build: {
    target: 'esnext'
  }
});
