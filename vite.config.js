import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

// Tauri ждёт фиксированный порт dev-сервера (tauri.conf.json build.devUrl)
export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  // src-tauri не следить: cargo пишет в target/ и на Windows держит .dll — наблюдатель падал с EBUSY
  server: { port: 1420, strictPort: true, watch: { ignored: ['**/src-tauri/**'] } },
  build: { target: 'es2022', assetsInlineLimit: 0, sourcemap: false },
});
