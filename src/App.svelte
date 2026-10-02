<script>
  import { onMount, tick } from 'svelte';
  import { app, WINDOW_SIZE, switchScene, errorHandle, t } from './lib/app.svelte.js';
  import { call } from './lib/backend.js';
  import { applyTheme } from './lib/theme.js';
  import Login from './scenes/Login.svelte';
  import ServerMenu from './scenes/ServerMenu.svelte';
  import Settings from './scenes/Settings.svelte';
  import Update from './scenes/Update.svelte';
  import Debug from './scenes/Debug.svelte';
  import { initGameEvents } from './lib/game.js';
  import { i18n } from './lib/i18n.svelte.js';
  import Notifications from './components/Notifications.svelte';
  import Processing from './components/Processing.svelte';
  import DevRef from './components/DevRef.svelte';
  import Dialog from './components/Dialog.svelte';
  import AddAccount from './components/AddAccount.svelte';
  import SelfUpdate from './components/SelfUpdate.svelte';
  import { sceneIn, sceneOut } from './lib/transitions.js';

  const scenes = { login: Login, servermenu: ServerMenu, settings: Settings, update: Update, debug: Debug };
  let ready = $state(false);

  onMount(async () => {
    try {
      initGameEvents();
      const init = await call('app_init');
      // новая версия лаунчера - ставим её до входа (LauncherSelfUpdate.checkAndUpdate)
      app.selfUpdate = await call('self_update_check').catch(() => null);
      app.settings = init.settings;
      app.javaVersion = init.javaVersion || '21';
      app.updatesDir = init.updatesDir || '';
      i18n.locale = app.settings.locale || 'RUSSIAN';
      applyTheme(app.settings);
      const params = new URLSearchParams(location.search);
      const start = import.meta.env.DEV && params.get('scene');
      if (start && scenes[start]) {
        if (start !== 'login') {
          app.user = { username: 'substanc1', uuid: '71a23136-c7ab-3271-bfe9-93dbe3c8f781' };
          app.profiles = await call('profiles_list');
        }
        app.scene = start;
      }
      app.quickAccounts = init.quickAccounts || [];
      await switchScene(app.scene, false);
    } catch (e) {
      errorHandle(e);
    }
    ready = true;
    // окно создаётся скрытым (tauri.conf visible:false) - показываем после первой отрисовки, без белой вспышки.
    // Не через requestAnimationFrame: WebKit (macOS/Linux) в скрытом окне его не вызывает - окно так и не появлялось
    await tick();
    call('window_show');
  });

</script>

{#if ready}
  <div class="window" class:freeze={import.meta.env.DEV && new URLSearchParams(location.search).has('freeze')}
       style="width:{WINDOW_SIZE[0]}px;height:{WINDOW_SIZE[1]}px">
    <div class="sceneWrap" class:blurred={app.blurred} class:calm={app.settings && app.settings.reduceEffects}>
      <!-- каждый экран - свой слой: новый проявляется поверх старого (lib/transitions.js) -->
      {#key app.scene + ':' + app.sceneNonce}
        <div class="sceneLayer" in:sceneIn out:sceneOut>
          {#if scenes[app.scene]}
            {@const Scene = scenes[app.scene]}
            <Scene />
          {:else}
            <div class="todo">{app.scene}</div>
          {/if}
        </div>
      {/key}
    </div>
    {#if app.selfUpdate}<SelfUpdate />{/if}
    <AddAccount />
    <Dialog />
    <Processing />
    <Notifications />
  </div>
  {#if import.meta.env.DEV}<DevRef />{/if}
{/if}

<style>
  .sceneWrap { position: absolute; inset: 0; transition: filter .28s ease-out; }
  .sceneWrap.calm { transition: none; }
  .sceneLayer { position: absolute; inset: 0; z-index: 1; transform-origin: 50% 50%; }
  /* FxStage.disable(): GaussianBlur(150) на сцене под оверлеем */
  .sceneWrap.blurred { filter: blur(50px); }
  /* dev: ?freeze - анимации стоят в исходном положении, как на статичных эталонах JavaFX */
  .freeze :global(*) { animation: none !important; }
  .todo { display: grid; place-items: center; height: 100%; font-family: 'JetBrains Mono'; color: var(--text-secondary); }
</style>
