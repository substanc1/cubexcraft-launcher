// Общее состояние лаунчера (аналог JavaFXApplication + FxStage: текущая сцена, оверлеи, уведомления).
import { call, inTauri } from './backend.js';
import { t, requestError } from './i18n.svelte.js';

// размеры сцен = prefWidth/prefHeight корневых FXML старого лаунчера (окно подгоняется, как stage.sizeToScene())
export const SCENE_SIZE = {
  login: [930, 560],
  servermenu: [1246, 640],
  settings: [930, 560],
  update: [930, 560],
  debug: [930, 560],
};

export const app = $state({
  scene: 'login',
  sceneFlow: ['login'],
  settings: null,
  user: null,
  profiles: [],
  selected: null,
  javaVersion: '21',
  processing: { visible: false, text: '', error: false },
  notifications: [],
  dialog: null, // { kind: 'apply'|'info', header, text, onApply, onDeny }
  blurred: false,
  quickAccounts: [], // [{ login, username }] - до 3, для меню быстрой смены
  addAccountOpen: false,
  playing: null, // { profile, error } - текущий запуск
  gameLog: [], // строки лога игры (экран debug)
  gameRunning: false,
  selfUpdate: null, // найденная новая версия лаунчера
});

let notifSeq = 0;
export function notify(head, message) {
  const id = ++notifSeq;
  app.notifications.push({ id, head, message });
  setTimeout(() => {
    const i = app.notifications.findIndex(n => n.id === id);
    if (i >= 0) app.notifications.splice(i, 1);
  }, 3000);
}

// FxComponent.errorHandle: ошибка запроса -> уведомление "Error" с переводом
export function errorHandle(e) {
  const msg = typeof e === 'string' ? e : (e && (e.message || e.toString())) || String(e);
  notify('Error', requestError(msg));
}

// ProcessingOverlay.processRequest: показать карточку "ЗАГРУЗКА..." с описанием, на успехе скрыть сразу,
// на ошибке - показать ошибку в карточке и скрыть через 1.5 с
let processingUse = 0;
export async function processRequest(text, promise, { onError } = {}) {
  processingUse++;
  app.processing = { visible: true, text, error: false };
  app.blurred = true;
  try {
    const r = await promise;
    if (--processingUse === 0) hideProcessing();
    return r;
  } catch (e) {
    if (onError) onError(e);
    else errorHandle(e);
    const shown = typeof e === 'string' ? e : String(e && (e.message || e));
    app.processing = { visible: true, text: shown, error: true };
    await new Promise(r => setTimeout(r, 1500));
    if (--processingUse === 0) hideProcessing();
    throw e;
  }
}
function hideProcessing() {
  app.processing = { ...app.processing, visible: false };
  app.blurred = false;
}

export async function switchScene(name, addToFlow = true) {
  if (addToFlow) app.sceneFlow.push(name);
  app.scene = name;
  const [w, h] = SCENE_SIZE[name] || SCENE_SIZE.login;
  await call('window_set_size', { width: w, height: h });
}

export async function back() {
  if (app.sceneFlow.length <= 1) return;
  app.sceneFlow.pop();
  await switchScene(app.sceneFlow[app.sceneFlow.length - 1], false);
}

export function showApplyDialog(header, text, onApply, onDeny) {
  app.dialog = { kind: 'apply', header, text, onApply, onDeny };
}

export async function saveSettings() {
  if (app.settings) await call('settings_save', { settings: $state.snapshot(app.settings) });
}

// FxStage.enableMouseDrag: окно тянется за любую "пустую" область (не поле ввода/кнопку/ссылку)
export async function startDrag(e) {
  if (e.button !== 0) return;
  if (e.target.closest('input, textarea, button, a, select, [data-no-drag], .scroll')) return;
  if (!inTauri) return;
  const { getCurrentWindow } = await import('@tauri-apps/api/window');
  getCurrentWindow().startDragging();
}

export { t };
