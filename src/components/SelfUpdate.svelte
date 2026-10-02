<script>
  // overlay/selfupdate (LauncherUpdateApply): карточка «CUBEXCRAFT | Обновление лаунчера», полоса, «УСТАНОВКА...».
  // Показывается, если при старте найдена новая версия лаунчера; ядро качает её, ставит и перезапускает.
  import { onMount } from 'svelte';
  import { app, errorHandle } from '../lib/app.svelte.js';
  import { call, on } from '../lib/backend.js';
  import logo from '../assets/images/logo.png';

  let progress = $state(-1);
  onMount(async () => {
    const off = await on('selfupdate-progress', p => (progress = p.total > 0 ? p.done / p.total : -1));
    try {
      await call('self_update_apply', { release: app.selfUpdate });
    } catch (e) {
      errorHandle(e);
      app.selfUpdate = null;
    }
    off();
  });
</script>

<div class="su-overlay">
  <div class="su-card">
    <div class="su-head">
      <img src={logo} alt="" height="20" />
      <span class="su-brand">CUBEXCRAFT</span>
      <span class="su-sep"></span>
      <span class="su-text">Обновление лаунчера</span>
    </div>
    <div class="su-bar">
      {#if progress < 0}<div class="su-run"></div>{:else}<div class="su-fill" style="width:{progress * 100}%"></div>{/if}
    </div>
    <div class="su-desc">УСТАНОВКА...</div>
  </div>
</div>

<style>
  .su-overlay { position: absolute; inset: 0; z-index: 70; display: grid; place-items: center; background: var(--base); }
  .su-card { width: 460px; padding: 16px 18px; background: var(--panel); border: 1px solid var(--border-dark); box-shadow: 0 12px 27px #000000aa; }
  .su-head { display: flex; align-items: center; gap: 10px; }
  .su-head img { height: 20px; width: auto; }
  .su-brand { font-family: 'Unbounded ExtraBold'; font-size: 12px; color: var(--text-hover); }
  .su-sep { width: 2px; height: 16px; border-left: 1px solid var(--text-secondary); }
  .su-text { font-family: 'JetBrains Mono'; font-size: 11.5px; color: var(--yellow); }
  .su-bar { margin-top: 14px; height: 6px; position: relative; overflow: hidden; background: var(--panel-flat); border: 1px solid var(--border-dark); }
  .su-fill { height: 100%; background: var(--red); box-shadow: 0 0 12px var(--glow-color); }
  .su-run { position: absolute; top: 0; bottom: 0; width: 25%; background: var(--red); animation: su-run 1.4s ease-in-out infinite; }
  @keyframes su-run { 0% { left: -25%; } 100% { left: 100%; } }
  .su-desc { margin-top: 10px; font-family: 'JetBrains Mono'; font-size: 11px; color: var(--text-secondary); }
</style>
