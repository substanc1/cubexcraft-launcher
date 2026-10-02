<script>
  // ServerMenuScene.showSwitchAccountMenu: контекстное меню под блоком профиля (minX, maxY + 6):
  // быстрые аккаунты (до 3), разделитель, «Добавить аккаунт» / «Лимит аккаунтов (3) исчерпан»
  import { app, t, processRequest, switchScene, errorHandle } from '../lib/app.svelte.js';
  import { call } from '../lib/backend.js';
  let { anchor, onclose } = $props();

  const list = $derived((app.quickAccounts || []).filter(a => a.login));
  const canAdd = $derived(list.length < 3);

  async function switchTo(acc) {
    onclose();
    try {
      const user = await processRequest(t('overlay.processing.text.auth'), call('auth_restore', { login: acc.login }));
      app.user = user;
      app.settings.login = acc.login;
      app.profiles = await processRequest(t('overlay.processing.text.profiles'), call('profiles_list'));
      app.selected = null;
      // экран серверов пересоздаётся (пинги/выбор), как reset() в старом лаунчере
      await switchScene('login', false);
      await switchScene('servermenu', false);
    } catch (e) {
      // сессия этого аккаунта умерла - на экран входа с его логином
      app.settings.login = acc.login;
      app.settings.hasSavedSession = false;
      await switchScene('login');
    }
  }

  function add() {
    onclose();
    app.addAccountOpen = true;
  }
</script>

<svelte:window onmousedown={e => { if (!e.target.closest('.qa-menu')) onclose(); }} onblur={onclose} />

<div class="qa-menu" style="left:{anchor.left}px;top:{anchor.bottom + 6}px">
  {#each list as acc}
    <button type="button" class="qa-item" onclick={() => switchTo(acc)}>{acc.login}</button>
  {/each}
  {#if list.length}<div class="qa-sep"></div>{/if}
  <button type="button" class="qa-item" onclick={add}>
    {canAdd ? t('scenes.servermenu.addAccount') : t('scenes.servermenu.addAccountFull')}
  </button>
</div>

<style>
  /* .context-menu.quickAccountMenu */
  .qa-menu {
    position: fixed; z-index: 60; min-width: 160px; padding: 4px;
    background: var(--panel); border: 1px solid var(--border-dark);
    box-shadow: 0 8px 16px #000000aa;
    display: flex; flex-direction: column;
  }
  .qa-item {
    text-align: left; padding: 8px 12px; background: transparent;
    font-family: 'JetBrains Mono'; font-size: 12px; line-height: 15.84px; color: var(--text); white-space: nowrap;
  }
  .qa-item:hover { background: var(--redTr); color: var(--text-hover); }
  .qa-sep { height: 0; margin: 4px 0; border-top: 1px solid var(--border-dark); }
</style>
