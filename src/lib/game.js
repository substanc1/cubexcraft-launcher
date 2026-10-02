// События запущенной игры (ядро: game-log / game-started / game-exit) - собираются глобально, чтобы экран лога
// (debug) показал всё с самого старта. Поведение как ProcessLogOutput: буфер ~256К символов; при выключенном
// «Debug-режиме» лаунчер сворачивается после старта игры и закрывается после её выхода.
import { app, back } from './app.svelte.js';
import { call, on } from './backend.js';

const MAX = 262144;
let chars = 0;

export function appendLog(text) {
  for (const line of String(text).split('\n')) {
    app.gameLog.push(line);
    chars += line.length + 1;
  }
  while (chars > MAX && app.gameLog.length > 1) {
    chars -= app.gameLog[0].length + 1;
    app.gameLog.shift();
  }
}

export function clearLog() {
  app.gameLog.length = 0;
  chars = 0;
}

export async function initGameEvents() {
  await on('game-log', p => appendLog(p.line));
  await on('game-started', () => {
    app.gameRunning = true;
    if (app.settings && app.settings.debugAllClients === false) call('window_minimize');
  });
  await on('game-exit', p => {
    app.gameRunning = false;
    appendLog(`Process finished with exit code ${p.code}`);
    if (app.settings && app.settings.debugAllClients === false) return call('window_close');
    // игру закрыли (или остановили кнопкой) - сразу обратно в лаунчер; при вылете остаёмся в логе, чтобы была видна ошибка
    if ((p.code === 0 || p.killed) && app.scene === 'debug') {
      back();
      call('window_show');
    }
  });
}
