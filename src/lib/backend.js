// Мост интерфейс -> ядро (Rust, src-tauri). В обычном браузере (vite dev, проверка вёрстки) вместо ядра -
// имитация с правдоподобными данными, чтобы экраны можно было проверять без сборки exe.
const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

let tauriCore = null;
let tauriEvent = null;
async function core() {
  if (!tauriCore) tauriCore = await import('@tauri-apps/api/core');
  return tauriCore;
}

export async function call(cmd, args = {}) {
  if (isTauri) return (await core()).invoke(cmd, args);
  const fn = mock[cmd];
  if (!fn) throw new Error('mock: no command ' + cmd);
  return fn(args);
}

// события ядра (прогресс загрузки, вывод игры): handler(payload); возвращает функцию отписки
export async function on(event, handler) {
  if (isTauri) {
    if (!tauriEvent) tauriEvent = await import('@tauri-apps/api/event');
    return tauriEvent.listen(event, e => handler(e.payload));
  }
  mockListeners[event] = mockListeners[event] || new Set();
  mockListeners[event].add(handler);
  return () => mockListeners[event].delete(handler);
}

export const inTauri = isTauri;

// ---------------------------------------------------------------- имитация ядра (только браузер)
const mockListeners = {};
const emit = (event, payload) => (mockListeners[event] || []).forEach(h => h(payload));
if (!isTauri) window.__mockEmit = emit; // проверка сценариев из консоли: __mockEmit('game-exit', { code: 0 })
const sleep = ms => new Promise(r => setTimeout(r, ms));
const params = new URLSearchParams(typeof location !== 'undefined' ? location.search : '');

const mockProfiles = [
  {
    uuid: 'b5bed500-0516-4b14-9f63-df7285b3bdf5', title: 'CubeXCraft', version: '1.21.1', dir: 'CubeXCraft',
    info: 'CubeXCraft — приватный сервер на NeoForge 1.21.1. Проект находится в стадии активной разработки: дорабатываем моды, механики и лаунчер перед первым публичным запуском.',
    servers: [{ name: 'CubeXCraft', serverAddress: 'play.cubexcraft.site', serverPort: 25565 }], properties: {},
  },
  {
    uuid: '2c1f0a5e-0000-4000-8000-000000000002', title: 'OneBlock', version: '1.21.1', dir: 'proj-oneblock',
    info: 'Create: OneBlock', servers: [{ name: 'OneBlock', serverAddress: 'play.cubexcraft.site', serverPort: 25591 }], properties: {},
  },
];

const mockSettings = {
  login: params.has('login') ? params.get('login') : 'substanc1', autoAuth: false, savePassword: true, hasSavedSession: params.has('saved'),
  lastProfile: null, quickAccounts: [], ram: 0, fullScreen: false, autoEnter: false, debugAllClients: true,
  reduceEffects: params.has('noeffects'), customColors: {}, updatesDir: 'C:\\Users\\Player\\AppData\\Roaming\\CubeXCraft\\updates',
};

const mock = {
  async app_init() {
    await sleep(150);
    return { settings: mockSettings, quickAccounts: [{ login: 'substanc1', username: 'substanc1' }], version: 'dev', javaVersion: '21', platform: 'windows-x64', updatesDir: mockSettings.updatesDir };
  },
  async auth_login({ login, password }) {
    await sleep(600);
    if (password === 'bad') throw 'auth.wrongpassword';
    return { username: login || 'Player', uuid: '71a23136-c7ab-3271-bfe9-93dbe3c8f781', skin: null };
  },
  async auth_restore() {
    await sleep(400);
    if (!mockSettings.hasSavedSession) throw 'auth.invalidtoken';
    return { username: mockSettings.login, uuid: '71a23136-c7ab-3271-bfe9-93dbe3c8f781', skin: null };
  },
  async auth_logout() { await sleep(200); },
  async auth_remember() { return [{ login: 'substanc1', username: 'substanc1' }]; },
  async updates_dir_pick() { return null; },
  async updates_dir_change({ path }) { return path; },
  async open_path() {},
  async self_update_check() { return null; },
  async profiles_list() { await sleep(300); return mockProfiles; },
  async server_ping({ host, port }) {
    await sleep(250 + Math.random() * 300);
    return port === 25565 ? { online: 3, max: 60 } : { online: 0, max: 20 };
  },
  async settings_save() {},
  async window_minimize() {},
  async window_show() {},
  async timing_mark() {},
  async window_close() { window.close(); },
  async window_drag() {},
  async open_url({ url }) { window.open(url, '_blank'); },
  async avatar_get() { return null; },
  async game_play({ uuid }) {
    const total = 296 * 1048576;
    for (let i = 0; i <= 20; i++) {
      await sleep(120);
      emit('sync-progress', { phase: i < 3 ? 'check' : 'download', done: (total * i) / 20, total, file: 'mods/example.jar', speed: 12.4e6 });
    }
    emit('game-log', { line: '[mock] Minecraft запущен' });
    setTimeout(() => emit('game-started', { pid: 1234 }));
    return { pid: 1234 };
  },
  async game_kill() { emit('game-exit', { code: -1, killed: true }); },
};
