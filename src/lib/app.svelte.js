// Общее состояние лаунчера (аналог JavaFXApplication + FxStage: текущая сцена, оверлеи, уведомления).
import { call, inTauri } from './backend.js';
import { t, requestError } from './i18n.svelte.js';

// размеры сцен = prefWidth/prefHeight корневых FXML старого лаунчера (окно подгоняется, как stage.sizeToScene())
// одно окно на все экраны (раньше вход/настройки/загрузка/лог были 930x560 и окно меняло размер при каждом переходе)
export const WINDOW_SIZE = [1246, 640];

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
  navDir: 1, // направление последнего перехода: 1 - вперёд, -1 - назад (для анимации)
  sceneNonce: 0, // +1 = пересоздать текущий экран (reloadScene)
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
// на ошибке - показать ошибку в карточке и скрыть через 1.5 с.
// Быстрый ответ (ядро запросило заранее, при старте) - без мигания: карточка появляется, только если ждать > 150 мс
let processingUse = 0;
export async function processRequest(text, promise, { onError } = {}) {
  processingUse++;
  const show = () => {
    app.processing = { visible: true, text, error: false };
    app.blurred = true;
  };
  let timer = null;
  if (app.processing.visible) show();
  else timer = setTimeout(show, 150);
  try {
    const r = await promise;
    clearTimeout(timer);
    if (--processingUse === 0) hideProcessing();
    return r;
  } catch (e) {
    clearTimeout(timer);
    app.blurred = true;
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
  app.navDir = addToFlow ? 1 : -1;
  app.scene = name;
}

// пересоздать текущий экран (смена аккаунта: пинги/выбор сервера заново), как reset() в старом лаунчере
export function reloadScene() {
  app.navDir = 1;
  app.sceneNonce++;
}

// FxStage.back: экраны с isDisableReturnBack (update) при возврате пропускаются - из лога игры назад в меню
const NO_RETURN_BACK = new Set(['update']);

export async function back() {
  let target;
  do {
    if (app.sceneFlow.length <= 1) return;
    app.sceneFlow.pop();
    target = app.sceneFlow[app.sceneFlow.length - 1];
  } while (NO_RETURN_BACK.has(target));
  await switchScene(target, false);
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
