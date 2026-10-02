// События запущенной игры (ядро: game-log / game-started / game-exit) - собираются глобально, чтобы экран лога
// (debug) показал всё с самого старта; разбор строк и буфер - gamelog.svelte.js. При выключенном «Debug-режиме»
// лаунчер сворачивается после старта игры и закрывается после её выхода.
import { app, back } from './app.svelte.js';
import { call, on } from './backend.js';
import { appendLog, clearLog, log } from './gamelog.svelte.js';

export { appendLog, clearLog };

export async function initGameEvents() {
  await on('game-log', p => appendLog(p.line));
  await on('game-started', () => {
    app.gameRunning = true;
    log.exit = null;
    if (app.settings && app.settings.debugAllClients === false) call('window_minimize');
  });
  await on('game-exit', p => {
    app.gameRunning = false;
    log.exit = { code: p.code, killed: !!p.killed, saved: p.saved || null };
    appendLog(`Process finished with exit code ${p.code}`);
    if (app.settings && app.settings.debugAllClients === false) return call('window_close');
    // игру закрыли (или остановили кнопкой) - сразу обратно в лаунчер; при вылете остаёмся в логе, чтобы была видна ошибка
    if ((p.code === 0 || p.killed) && app.scene === 'debug') {
      back();
      call('window_show');
    }
  });
}
