// Кнопка ИГРАТЬ: экран обновления (загрузка/проверка файлов) -> запуск -> экран лога игры (debug).
// Как ServerMenuScene.play: switchScene(updateScene) -> downloadProfile -> debugScene.run
import { app, switchScene, back, errorHandle } from './app.svelte.js';
import { call } from './backend.js';
import { clearLog } from './game.js';

export async function play(profile) {
  const ps = (app.settings.profiles && app.settings.profiles[profile.uuid]) || {};
  app.playing = { profile, error: null };
  clearLog(); // DebugScene.isResetOnShow: лог прошлого запуска не тянется в новый
  await switchScene('update');
  try {
    await call('game_play', {
      uuid: profile.uuid,
      options: { ram: ps.ram || 0, fullScreen: !!ps.fullScreen, autoEnter: !!ps.autoEnter },
    });
    await switchScene('debug');
  } catch (e) {
    if (e !== 'cancelled') {
      errorHandle(e);
      app.playing = { profile, error: String(e) };
    }
    await back(); // update снимается со стека, возвращаемся в меню серверов
  }
}

// Кнопка «Проверить целостность»: тот же экран прогресса, что у ИГРАТЬ, но без запуска игры.
// build_verify сверяет библиотеки, assets и клиент (sha1), лишние файлы клиента удаляет; возвращает статистику.
export async function verifyBuild(profile) {
  app.playing = { profile, error: null, verify: true };
  await switchScene('update');
  try {
    const st = await call('build_verify', { uuid: profile.uuid });
    await back();
    return st;
  } catch (e) {
    if (e !== 'cancelled') {
      errorHandle(e);
      app.playing = { profile, error: String(e), verify: true };
    }
    await back(); // update снимается со стека, возвращаемся в меню серверов
    return null;
  }
}
