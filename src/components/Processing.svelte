<script>
  // overlay/processing: затемнение rgba(4,4,12,.92) поверх размытой сцены, свои кнопки свернуть/закрыть
  // (right 29, top 34), карточка 460px по центру: лого + CUBEXCRAFT | ЗАГРУЗКА..., бегущая полоса, описание
  import { app, t } from '../lib/app.svelte.js';
  import { call } from '../lib/backend.js';
  import IconButton from './IconButton.svelte';
  import logo from '../assets/images/logo.png';
  import { popIn, popOut, fadeQuick } from '../lib/transitions.js';

  // ожидание подтверждения в Telegram: отсчёт до конца запроса (app.processing.until) и «ОТМЕНА»
  let now = $state(Date.now());
  $effect(() => {
    if (!app.processing.until) return;
    now = Date.now();
    const id = setInterval(() => (now = Date.now()), 1000);
    return () => clearInterval(id);
  });
  const left = $derived.by(() => {
    const s = Math.max(0, Math.round(((app.processing.until || 0) - now) / 1000));
    return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, '0')}`;
  });
  let cancelling = $state(false);
  function cancel() {
    if (cancelling) return;
    cancelling = true;
    Promise.resolve(app.processing.onCancel && app.processing.onCancel()).finally(() => (cancelling = false));
  }
</script>

{#if app.processing.visible}
  <div class="overlay" in:fadeQuick out:fadeQuick={{ duration: 160 }}>
    <div class="dim"></div>
    <div class="controls">
      <IconButton glyph="—" tip={t('scenes.servermenu.minimizeTooltip')} onclick={() => call('window_minimize')} />
      <IconButton glyph="✕" kind="exit" tip={t('scenes.servermenu.exitTooltip')} onclick={() => call('window_close')} />
    </div>
    <div class="center">
      <div class="card" in:popIn={{ delay: 40 }} out:popOut>
        <div class="cardHeader">
          <img src={logo} alt="" height="20" />
          <span class="brand">CUBEXCRAFT</span>
          <span class="sep"></span>
          <span class="headerText">{t('overlay.processing.processing.description')}</span>
        </div>
        <div class="bar"><div class="run"></div></div>
        <div class="description" class:error={app.processing.error}>{app.processing.text}</div>
        {#if !app.processing.error && (app.processing.until || app.processing.onCancel)}
          <div class="waitRow">
            {#if app.processing.until}<span class="left">{left}</span>{/if}
            <span class="grow"></span>
            {#if app.processing.onCancel}
              <button type="button" class="cancel" disabled={cancelling} onclick={cancel}>{t('scenes.loginpassword.header-button-text')}</button>
            {/if}
          </div>
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .overlay { position: absolute; inset: 0; z-index: 40; }
  .dim { position: absolute; inset: 0; background: rgba(4, 4, 12, 0.92); }
  .controls { position: absolute; right: 29px; top: 34px; display: flex; gap: 10px; }
  .center { position: absolute; inset: 0; display: grid; place-items: center; pointer-events: none; }
  .card {
    width: 460px; pointer-events: auto;
    background: var(--panel);
    border: 1px solid var(--border-dark);
    padding: 16px 18px;
    box-shadow: 0 12px 27px #000000aa;
  }
  .cardHeader { display: flex; align-items: center; gap: 10px; }
  .cardHeader img { height: 20px; width: auto; }
  .brand { font-family: 'Unbounded ExtraBold'; font-size: 12px; color: var(--text-hover); }
  .sep { width: 2px; height: 16px; border-left: 1px solid var(--text-secondary); border-right: 1px solid transparent; opacity: .9; }
  .headerText { font-family: 'JetBrains Mono'; font-size: 11.5px; color: var(--yellow); }
  .bar {
    margin-top: 14px; height: 6px; position: relative; overflow: hidden;
    background: var(--panel-flat); border: 1px solid var(--border-dark);
  }
  /* ProgressBar progress=-1: бегущий отрезок */
  .run {
    position: absolute; top: 0; bottom: 0; width: 25%;
    background: var(--red); box-shadow: 0 0 16px var(--glow-color);
    animation: run 1.4s ease-in-out infinite;
  }
  @keyframes run { 0% { left: -25%; } 100% { left: 100%; } }
  .description {
    margin-top: 10px; min-height: 14px;
    font-family: 'JetBrains Mono'; font-size: 11px; color: var(--text-secondary);
    overflow-wrap: anywhere;
  }
  .description.error { color: var(--red); }
  .waitRow { margin-top: 12px; display: flex; align-items: center; gap: 10px; }
  .grow { flex: 1; }
  .left { font-family: 'JetBrains Mono'; font-size: 11.5px; color: var(--yellow); }
  /* как .aa-submit окна «Добавить аккаунт», только красная */
  .cancel {
    height: 30px; padding: 0 16px; background: transparent; border: 1px solid var(--border-dark); color: var(--text-secondary);
    font-family: 'JetBrains Mono Bold'; font-size: 11px; line-height: 14.52px; cursor: pointer;
  }
  .cancel:hover { border-color: var(--red); color: var(--text-hover); }
  .cancel:disabled { opacity: .5; cursor: default; }
</style>
