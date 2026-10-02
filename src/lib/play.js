// Кнопка ИГРАТЬ: экран обновления (загрузка/проверка файлов) -> запуск -> экран лога игры (debug).
// Как ServerMenuScene.play: switchScene(updateScene) -> downloadProfile -> debugScene.run
import { app, switchScene, errorHandle } from './app.svelte.js';
import { call } from './backend.js';

export async function play(profile) {
  const ps = (app.settings.profiles && app.settings.profiles[profile.uuid]) || {};
  app.playing = { profile, error: null };
  await switchScene('update');
  try {
    await call('game_play', {
      uuid: profile.uuid,
      options: { ram: ps.ram || 0, fullScreen: !!ps.fullScreen, autoEnter: !!ps.autoEnter },
    });
    await switchScene('debug');
  } catch (e) {
    if (e === 'cancelled') {
      await switchScene('servermenu');
      return;
    }
    errorHandle(e);
    app.playing = { profile, error: String(e) };
    await switchScene('servermenu');
  }
}
