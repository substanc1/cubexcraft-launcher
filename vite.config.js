import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

// Tauri ждёт фиксированный порт dev-сервера (tauri.conf.json build.devUrl)
export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  server: { port: 1420, strictPort: true },
  build: { target: 'es2022', assetsInlineLimit: 0, sourcemap: false },
});
