<script>
  // scenes/debug/debug.fxml + DebugScene/ProcessLogOutput: живой лог игры, копировать, убить процесс, назад;
  // закрытие при запущенной игре - подтверждение (forceExitDialog)
  import { app, t, back, startDrag, showApplyDialog } from '../lib/app.svelte.js';
  import { call } from '../lib/backend.js';
  import IconButton from '../components/IconButton.svelte';
  import { fxscroll } from '../lib/fxscroll.js';

  let out;
  $effect(() => {
    app.gameLog.length;
    queueMicrotask(() => out && (out.scrollTop = out.scrollHeight));
  });

  function copy() {
    navigator.clipboard && navigator.clipboard.writeText(app.gameLog.join('\n'));
  }
  function exit() {
    if (!app.gameRunning) return call('window_close');
    showApplyDialog(t('scenes.debug.forceExitDialog.header'), t('scenes.debug.forceExitDialog.description'), () => call('window_close'));
  }
</script>

<div class="dbg-layout bg-image" onmousedown={startDrag} role="presentation">
  <div class="dbg-header">
    <span class="dbg-back"><IconButton glyph="←" tip={t('scenes.debug.backTooltip')} onclick={back} /></span>
    <span class="dbg-title">{t('scenes.debug.headingDebug')}</span>
    <div class="dbg-controls">
      <IconButton glyph="⧉" tip={t('scenes.debug.copyTooltip')} onclick={copy} />
      <IconButton glyph="■" kind="kill" tip={t('scenes.debug.killTooltip')} onclick={() => call('game_kill')} />
      <IconButton glyph="—" tip={t('scenes.servermenu.minimizeTooltip')} onclick={() => call('window_minimize')} />
      <IconButton glyph="✕" kind="exit" tip={t('scenes.servermenu.exitTooltip')} onclick={exit} />
    </div>
  </div>
  <div class="dbg-outputDetail">
    <div class="dbg-output scroll" bind:this={out} use:fxscroll={{ right: -17, top: 109, bottom: 109, width: 12 }}>{app.gameLog.join('\n')}</div>
  </div>
</div>

<style>
  .dbg-layout { position: relative; width: 930px; height: 560px; overflow: hidden; }
  .dbg-header { position: absolute; left: 0; right: 0; top: 0; height: 100px; border: 1px solid transparent; border-bottom-color: var(--border-dark); }
  .dbg-back { position: absolute; left: 28px; top: 34px; }
  .dbg-title {
    position: absolute; left: 73px; top: 34px; height: 32px; display: flex; align-items: center;
    font-family: 'Unbounded SemiBold'; font-size: 18px; line-height: 22.32px; color: var(--text-hover); white-space: nowrap;
  }
  .dbg-controls { position: absolute; right: 29px; top: 34px; display: flex; gap: 10px; }
  /* #output-detail: StackPane 50/50/110/24, фон panel-flat, рамка */
  .dbg-outputDetail {
    position: absolute; left: 50px; right: 50px; top: 110px; bottom: 24px;
    background: var(--panel-flat); border: 1px solid var(--border-dark);
  }
  .dbg-output {
    position: absolute; inset: 0; overflow-y: auto; padding: 10px; white-space: pre-wrap; overflow-wrap: anywhere;
    user-select: text; -webkit-user-select: text; cursor: text;
    font-family: 'JetBrains Mono'; font-size: 12px; line-height: 15.84px; color: var(--text);
  }
</style>
