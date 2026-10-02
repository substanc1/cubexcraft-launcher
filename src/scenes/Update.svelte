<script>
  // scenes/update/update.fxml + UpdateScene/VisualDownloader: скорость (как в оригинале - мегабиты/с, подпись МБ/С),
  // объём X/Y MB, журнал фаз/этапов, полоса прогресса, кнопка отмены.
  import './update.css';
  import { onMount, onDestroy } from 'svelte';
  import { app, t, startDrag } from '../lib/app.svelte.js';
  import { call, on } from '../lib/backend.js';
  import IconButton from '../components/IconButton.svelte';
  import logo from '../assets/images/logo.png';

  let speed = $state('0');
  let volume = $state('');
  let progress = $state(0);
  let log = $state([]);
  let error = $state(false);
  let logEl;
  let stage = null;
  let phase = null;
  let last = { time: 0, done: 0 };
  let off = null;

  const PHASE = { java: 'java', libs: 'client', assets: 'assets', client: 'client' };

  function addLog(s) {
    log.push(s);
    queueMicrotask(() => logEl && (logEl.scrollTop = logEl.scrollHeight));
  }

  onMount(async () => {
    off = await on('sync-progress', p => {
      const ph = PHASE[p.stage] || p.stage;
      if (p.stage !== stage) {
        // libs (свои библиотеки лаунчера) - часть фазы клиента, отдельно не объявляем
        if (!(p.stage === 'libs' || (p.stage === 'client' && stage === 'libs'))) addLog(t('scenes.update.phase.' + ph).replace(/^\n/, ''));
        else if (p.stage === 'libs') addLog(t('scenes.update.phase.client'));
        stage = p.stage;
        phase = null;
      }
      if (p.phase !== phase) {
        phase = p.phase;
        const key = { check: 'hashing', download: 'download', done: 'done.part' }[p.phase];
        if (key && !(p.phase === 'download' && p.total === 0)) addLog(t('scenes.update.stage.' + key));
        last = { time: performance.now(), done: 0 };
        progress = 0;
      }
      if (p.phase === 'download' || p.stage === 'java') {
        progress = p.total > 0 ? p.done / p.total : 0;
        const now = performance.now();
        if (now - last.time >= 130) {
          const bytesSpeed = ((p.done - last.done) / (now - last.time)) * 1000;
          speed = ((bytesSpeed * 8) / 1e6).toFixed(1);
          volume = `${(p.done / 1048576).toFixed(1)}/${(p.total / 1048576).toFixed(1)} MB`;
          last = { time: now, done: p.done };
        }
      } else if (p.phase === 'check') {
        progress = p.total > 0 ? p.done / p.total : 0;
      }
    });
  });
  onDestroy(() => off && off());

  // ошибка загрузки (play.js ставит app.playing.error) - как UpdateScene.errorHandle
  $effect(() => {
    if (app.playing && app.playing.error) {
      error = true;
      addLog(`Exception: ${app.playing.error}`);
    }
  });

  function cancel() {
    call('sync_cancel');
  }
</script>

<div class="up-layout bg-image" onmousedown={startDrag} role="presentation">
  <div class="up-header">
    <div class="up-title">
      <img src={logo} alt="" />
      <span>{t('scenes.update.headingDialog')}</span>
    </div>
    <div class="up-controls">
      <IconButton glyph="✕" kind="cancel" tip={t('scenes.update.cancelTooltip')} onclick={cancel} />
      <IconButton glyph="—" tip={t('scenes.servermenu.minimizeTooltip')} onclick={() => call('window_minimize')} />
      <IconButton glyph="✕" kind="exit" tip={t('scenes.servermenu.exitTooltip')} onclick={() => call('window_close')} />
    </div>
  </div>

  <div class="up-body">
    <div class="up-detail">
      {#if !error}
        <div class="up-speedOn">
          <div class="up-stat">
            <span class="up-speed">{speed}</span>
            <span class="up-speedText">МБ/С</span>
          </div>
          <div class="up-stat">
            <span class="up-volume">{volume}</span>
            <span class="up-caption">{t('scenes.update.volumeCaption')}</span>
          </div>
        </div>
      {:else}
        <span class="up-speedErr">ERR</span>
      {/if}
    </div>
    <div class="up-output scroll" bind:this={logEl}>{log.join('\n')}</div>
  </div>

  <div class="up-progressTrack">
    <div class="up-progress" class:error style="width:{Math.min(100, progress * 100)}%"></div>
  </div>

  <div class="up-footer">
    <span class="up-footerStatus">{t('scenes.update.footerStatus')}</span>
  </div>
</div>
