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
    uuid: 'b5bed500-0516-4b14-9f63-df7285b3bdf5', title: 'CubeXTech', version: '1.21.1', dir: 'CubeXCraft',
    info: 'CubeXTech — приватный сервер на NeoForge 1.21.1. Проект находится в стадии активной разработки: дорабатываем моды, механики и лаунчер перед первым публичным запуском.',
    servers: [{ name: 'CubeXCraft', serverAddress: 'play.cubexcraft.site', serverPort: 25565 }], properties: {},
  },
  {
    uuid: '2c1f0a5e-0000-4000-8000-000000000002', title: 'OneCube', version: '1.21.1', dir: 'proj-oneblock',
    info: 'Create: OneCube', servers: [{ name: 'OneCube', serverAddress: 'play.cubexcraft.site', serverPort: 25591 }], properties: {},
  },
];

const mockSettings = {
  login: params.has('login') ? params.get('login') : 'substanc1', autoAuth: false, savePassword: true, hasSavedSession: params.has('saved'),
  lastProfile: null, quickAccounts: [], ram: 0, fullScreen: false, autoEnter: false, debugAllClients: true,
  reduceEffects: params.has('noeffects'), customColors: {}, updatesDir: 'C:\\Users\\Player\\AppData\\Roaming\\CubeXCraft\\updates',
};

let mock2fa = null;

const mock = {
  async app_init() {
    await sleep(150);
    return { settings: mockSettings, quickAccounts: [{ login: 'substanc1', username: 'substanc1' }], version: 'dev', javaVersion: '21', platform: 'windows-x64', updatesDir: mockSettings.updatesDir };
  },
  async auth_login({ login, password }) {
    await sleep(600);
    if (password === 'bad') throw 'auth.wrongpassword';
    // пароль «2fa» - вход с подтверждением в Telegram (через 5 с «Это я»; «deny» - отказ)
    if (password === '2fa' || password === 'deny') {
      mock2fa = { login, deny: password === 'deny', cancelled: false };
      return { twofa: { challenge: 'mock', expiresIn: 180, bot: 'CubeXCraftBot' } };
    }
    return { username: login || 'Player', uuid: '71a23136-c7ab-3271-bfe9-93dbe3c8f781', skin: null };
  },
  async auth_2fa_wait() {
    const st = mock2fa;
    for (let i = 0; i < 50 && !st.cancelled; i++) await sleep(100);
    if (st.cancelled) throw '2fa.cancelled';
    if (st.deny) throw 'Вход отклонён в Telegram';
    return { username: st.login || 'Player', uuid: '71a23136-c7ab-3271-bfe9-93dbe3c8f781', skin: null };
  },
  async auth_2fa_cancel() {
    if (mock2fa) mock2fa.cancelled = true;
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
    emit('game-log', { line: 'C:\\Users\\Player\\AppData\\Roaming\\CubeXCraft\\cxl\\java\\bin\\java.exe -Xmx6144M -XX:+UseG1GC -cp ... cpw.mods.bootstraplauncher.BootstrapLauncher --username substanc1 --accessToken ********' });
    setTimeout(() => emit('game-started', { pid: 1234 }));
    setTimeout(() => mockGameLog(), 50);
    return { pid: 1234 };
  },
  async logs_open() {},
  async log_error() {},
  // друзья (только браузер): состояние в памяти, ответы как у сайта
  async friends_get() { return mockFriends.snap(); },
  async friends_action({ action, username, server }) { return mockFriends.act(action, username, server); },
  async notes_action({ action, id }) { return mockFriends.notes(action, id); },
  async presence_set() {},
  async game_kill() { emit('game-exit', { code: -1, killed: true }); },
};

const mockFriends = {
  friends: [
    { username: 'Minison434', uuid: '1', state: 'playing', server: 'CubeXTech', playingSec: 4320 },
    { username: 'v_kirsanov', uuid: '2', state: 'online', server: null, playingSec: null },
    { username: 'nullbyte', uuid: '3', state: 'away', server: null, playingSec: null },
    { username: 'm0rf', uuid: '4', state: 'offline', server: null, playingSec: null },
  ],
  requests: [{ username: 'ironwolf', uuid: '5', mutual: 2 }, { username: 'kate_tech', uuid: '6', mutual: 0 }],
  notifications: [
    { id: '3', kind: 'invite', actor: 'Minison434', server: 'CubeXTech', unread: true, agoSec: 180 },
    { id: '2', kind: 'accepted', actor: 'nullbyte', server: '', unread: true, agoSec: 60000 },
    { id: '1', kind: 'playing', actor: 'v_kirsanov', server: 'OneCube', unread: false, agoSec: 90000 },
  ],
  snap() { return JSON.parse(JSON.stringify({ friends: this.friends, requests: this.requests, notifications: this.notifications })); },
  async act(action, username, server) {
    await sleep(150);
    const name = String(username || '').trim();
    if (action === 'request') {
      if (name.toLowerCase() === 'nobody') throw 'Игрок не найден';
      return { result: 'sent', username: name, ...this.snap() };
    }
    if (action === 'accept') {
      this.requests = this.requests.filter(r => r.username !== name);
      this.friends.push({ username: name, uuid: name, state: 'online', server: null, playingSec: null });
    }
    if (action === 'decline') this.requests = this.requests.filter(r => r.username !== name);
    if (action === 'remove') this.friends = this.friends.filter(f => f.username !== name);
    return { username: name, ...this.snap() };
  },
  async notes(action, id) {
    await sleep(100);
    if (action === 'read') this.notifications.forEach(n => (n.unread = false));
    else this.notifications = this.notifications.filter(n => n.id !== id);
    return this.snap();
  },
};

// поток журнала игры для проверки экрана debug в браузере (?crash - в конце вылет с сохранённым журналом)
function mockGameLog() {
  const T = s => `[15:39:${String(10 + (s % 50)).padStart(2, '0')}]`;
  const lines = [];
  const info = (s, th, src, msg) => lines.push(`${T(s)} [${th}/INFO] [${src}/]: ${msg}`);
  for (let i = 0; i < 6; i++) info(0, 'main', 'cp.mo.mo.LaunchServiceHandler', `Launching target 'forgeclient' with arguments [--version, 1.21.1, --gameDir, C:/Users/Player/AppData/Roaming/CubeXCraft/updates/CubeXCraft]`.slice(0, 60 + i * 9));
  lines.push('WARNING: You are running with unsupported Java 21. To make full use of the Vector API, please update Apache Lucene.');
  lines.push(`${T(1)} [main/WARN] [mixin/]: Error loading class: com/buuz135/functionalstorage/world/EnderSavedData (java.lang.ClassNotFoundException: com.buuz135.functionalstorage.world.EnderSavedData)`);
  lines.push(`${T(1)} [main/WARN] [mixin/]: @Mixin target com.buuz135.functionalstorage.world.EnderSavedData was not found mixins.extendedae.json:MixinEnderSavedData from mod extendedae`);
  info(2, 'modloading-worker-0', 'co.ze.ne.st.StorageManager', "StorageManager: active backend is 'json'");
  info(2, 'modloading-worker-0', 'co.ze.ne.ch.AfkManager', 'Loaded AFK data for 0 players');
  info(2, 'modloading-worker-0', 'co.ze.ne.te.HomeManager', '[HomeManager] Config loaded — safetyCheck=true, maxHomes=5, warmup=3s, tpCooldown=0s, setCooldown=0s, delCooldown=0s, crossDimension=true');
  info(2, 'modloading-worker-0', 'Puzzles Lib', 'Constructing components for puzzleslib:client');
  info(2, 'modloading-worker-0', 'STDOUT', '[com.zerog.neoessentials.util.MessageUtil:logInfo:930]: [NeoEssentials-Lang] INFO: [Lang] Active language: en_us');
  for (let i = 0; i < 4; i++) info(3, 'modloading-worker-0', 'co.ze.ne.NeoEssentials', '═══════════════════════════════');
  for (const m of ['water', 'lava', 'milk', 'honey', 'oil', 'steam']) lines.push(`${T(4)} [modloading-sync-worker/WARN] [GTCEu/]: FluidStorageKey{gtceu:liquid} already has an associated fluid for material gtceu:${m}`);
  for (let i = 0; i < 160; i++) info(5 + (i >> 4), 'Render thread', i % 3 ? 'ne.mi.cl.re.mo.ModelManager' : 'GTCEu', `Loaded model batch ${i} (${(i * 37) % 900} models)`);
  lines.push(`${T(20)} [Render thread/ERROR] [ne.mi.cl.re.te.TextureAtlas/]: Using missing texture, unable to load gtceu:textures/block/casings/voltage/uv/side.png`);
  lines.push('java.io.FileNotFoundException: gtceu:textures/block/casings/voltage/uv/side.png');
  lines.push('\tat net.minecraft.server.packs.resources.ResourceProvider.lambda$getResourceOrThrow$1(ResourceProvider.java:22)');
  lines.push('\tat java.base/java.util.Optional.orElseThrow(Optional.java:403)');
  lines.push('\tat net.minecraft.client.renderer.texture.SpriteLoader.loadSprite(SpriteLoader.java:166)');
  lines.push('\tat java.base/java.util.concurrent.CompletableFuture$AsyncSupply.run(CompletableFuture.java:1768)');
  lines.push('\t... 4 more');
  info(21, 'Render thread', 'ne.mi.cl.Minecraft', 'Backend library: LWJGL version 3.3.4+7');
  lines.push(`${T(22)} [Render thread/WARN] [ne.mi.cl.Options/]: Skipping bad option: lastServer:`);
  info(23, 'Render thread', 'cubexcraft', 'CubexTitleScreen registered');
  let i = 0;
  const iv = setInterval(() => {
    for (let k = 0; k < 25 && i < lines.length; k++) emit('game-log', { line: lines[i++] });
    if (i >= lines.length) {
      clearInterval(iv);
      if (params.has('crash')) {
        emit('game-log', { line: `${T(30)} [Render thread/FATAL] [ne.mi.cl.Minecraft/]: Unreported exception thrown!` });
        emit('game-log', { line: 'java.lang.IllegalStateException: Rendersystem called from wrong thread' });
        emit('game-log', { line: '\tat com.mojang.blaze3d.systems.RenderSystem.assertOnRenderThread(RenderSystem.java:262)' });
        setTimeout(() => emit('game-exit', { code: 1, killed: false, saved: 'game-2026-10-02_13-50-12.log' }), 300);
      }
    }
  }, 40);
}
