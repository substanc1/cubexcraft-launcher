<script>
  // overlay/addaccount (AddAccountOverlay, CenterOverlay): по центру поверх размытой сцены. Перед входом текущий
  // аккаунт запоминается (QuickAccounts.rememberCurrent), новый становится текущим и первым в быстром списке.
  import { app, t, processRequest, reloadScene } from '../lib/app.svelte.js';
  import { call } from '../lib/backend.js';
  import IconButton from './IconButton.svelte';
  import { popIn, popOut } from '../lib/transitions.js';

  let login = $state('');
  let password = $state('');
  let error = $state(null);

  $effect(() => {
    if (app.addAccountOpen) {
      app.blurred = true;
      login = '';
      password = '';
      error = null;
    }
  });

  function close() {
    app.addAccountOpen = false;
    app.blurred = false;
  }

  async function submit() {
    if (!login.trim() || !password) {
      error = t('overlay.addaccount.emptyFields');
      return;
    }
    error = null;
    try {
      await call('auth_remember', { remember: true });
      const user = await processRequest(t('overlay.processing.text.auth'), call('auth_login', { login, password }), { onError: () => {} });
      app.quickAccounts = await call('auth_remember', { remember: true });
      app.user = user;
      app.settings.login = login;
      app.profiles = await processRequest(t('overlay.processing.text.profiles'), call('profiles_list'));
      app.selected = null;
      close();
      reloadScene();
    } catch (e) {
      error = t('overlay.addaccount.authFailed');
      app.blurred = true;
    }
  }
</script>

{#if app.addAccountOpen}
  <div class="aa-wrap">
    <div class="aa-dialog" in:popIn out:popOut>
      <div class="aa-head">
        <span class="aa-heading">{t('overlay.addaccount.header')}</span>
        <span class="aa-close"><IconButton glyph="✕" kind="close" tip={t('dialogs.apply.close-dialogTooltip')} onclick={close} /></span>
      </div>
      <div class="aa-body">
        <input class="aa-field" bind:value={login} placeholder={t('scenes.login.login')} spellcheck="false" onkeydown={e => e.key === 'Enter' && submit()} />
        <input class="aa-field" type="password" bind:value={password} placeholder={t('scenes.login.password')} onkeydown={e => e.key === 'Enter' && submit()} />
        {#if error}<span class="aa-error">{error}</span>{/if}
        <button type="button" class="aa-submit" onclick={submit}>{t('overlay.addaccount.submit')}</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .aa-wrap { position: absolute; inset: 0; z-index: 44; display: grid; place-items: center; }
  .aa-dialog {
    width: 380px; padding: 14px; display: flex; flex-direction: column; gap: 14px;
    background: var(--panel); border: 1px solid var(--border-dark); box-shadow: var(--dropshadow);
  }
  .aa-head { position: relative; height: 32px; }
  .aa-heading {
    position: absolute; left: 34px; right: 34px; top: 0; height: 32px; display: grid; place-items: center;
    font-family: 'Unbounded SemiBold'; font-size: 16px; line-height: 19.84px; color: var(--text-hover); white-space: nowrap;
  }
  .aa-close { position: absolute; right: 0; top: 0; }
  .aa-body { display: flex; flex-direction: column; gap: 10px; }
  /* .dialog .text-field */
  .aa-field {
    height: 46px; padding: 10px 12px; background: var(--panel-flat); border: 1px solid var(--border-dark);
    color: var(--text-hover); font-family: 'Manrope'; font-size: 13px;
  }
  .aa-field::placeholder { color: var(--text-secondary); opacity: 1; }
  .aa-field:focus { border-color: var(--green); }
  .aa-error { width: 350px; color: var(--red); font-family: 'Manrope'; font-size: 12px; line-height: 16.39px; }
  .aa-submit {
    height: 42px; background: transparent; border: 1px solid var(--green); color: var(--green);
    font-family: 'JetBrains Mono Bold'; font-size: 11px; line-height: 14.52px;
  }
  .aa-submit:hover { background: var(--greenTr); }
</style>
