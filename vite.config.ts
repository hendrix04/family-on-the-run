import { defineConfig, loadEnv } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

export default defineConfig(({ mode }) => ({
  base: './',
  build: { rollupOptions: { output: {
    hashCharacters: 'hex',
    entryFileNames: 'assets/app-[hash]-js',
    chunkFileNames: 'assets/chunk-[hash]-js',
    assetFileNames: asset => {
      const filename = asset.names[0] ?? 'asset';
      const extension = filename.split('.').pop()?.toLowerCase() ?? 'bin';
      const name = filename.replace(/\.[^.]+$/, '').toLowerCase().replace(/[^a-z0-9_-]/g, '-');
      return `assets/${name}-[hash]-${extension}`;
    },
  } } },
  plugins: [svelte()],
  server: {
    host: '127.0.0.1',
    port: 5173,
    strictPort: true,
    allowedHosts: loadEnv(mode, '.', 'VITE_').VITE_DEV_ALLOWED_HOST?.split(',') ?? [],
    proxy: {
      '/v1': { target: 'http://127.0.0.1:3000', ws: true },
    },
  },
}));
