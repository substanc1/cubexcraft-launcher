<script>
  // dialogs/apply/dialog.fxml (ApplyDialog) и dialogs/info (InfoDialog): по центру окна поверх размытой сцены
  // (MessageManager.initDialogInScene -> scene.disable()), 600px, рамка + отступ 14.
  import { app, t } from '../lib/app.svelte.js';
  import IconButton from './IconButton.svelte';

  function close(fn) {
    const d = app.dialog;
    app.dialog = null;
    app.blurred = false;
    if (fn) fn();
    else if (d && d.onDeny) d.onDeny();
  }
  $effect(() => {
    if (app.dialog) app.blurred = true;
  });
</script>

{#if app.dialog}
  {@const d = app.dialog}
  <div class="dlg-wrap">
    <div class="dlg">
      <div class="dlg-head">
        <span class="dlg-heading">{d.header}</span>
        <span class="dlg-close"><IconButton glyph="✕" kind="close" tip={t('dialogs.apply.close-dialogTooltip')} onclick={() => close()} /></span>
      </div>
      <div class="dlg-body">
        <div class="dlg-text">{d.text}</div>
        <div class="dlg-actions">
          {#if d.kind === 'info'}
            <button type="button" class="dlg-btn apply" onclick={() => close(d.onApply || (() => {}))}>{t('dialogs.info.info-button')}</button>
          {:else}
            <button type="button" class="dlg-btn apply" onclick={() => close(d.onApply || (() => {}))}>{t('dialogs.apply.apply-button')}</button>
            <span class="dlg-sep"></span>
            <button type="button" class="dlg-btn deny" onclick={() => close()}>{t('dialogs.apply.deny-button')}</button>
          {/if}
        </div>
      </div>
    </div>
  </div>
{/if}

<style>
  .dlg-wrap { position: absolute; inset: 0; z-index: 45; display: grid; place-items: center; }
  .dlg {
    width: 600px; padding: 14px; display: flex; flex-direction: column; gap: 10px;
    background: var(--panel); border: 1px solid var(--border-dark); box-shadow: var(--dropshadow);
  }
  .dlg-head { position: relative; height: 32px; }
  .dlg-heading {
    position: absolute; left: 42px; right: 42px; top: 0; height: 32px;
    display: grid; place-items: center; text-align: center;
    font-family: 'Unbounded SemiBold'; font-size: 16px; line-height: 19.84px; color: var(--text-hover);
  }
  .dlg-close { position: absolute; right: 0; top: 0; }
  .dlg-body { display: flex; flex-direction: column; gap: 20px; }
  .dlg-text {
    max-width: 578px; min-height: 37px; padding: 10px 15px; text-align: center;
    background: var(--panel-flat); color: var(--text-secondary);
    font-family: 'Manrope'; font-size: 12px; line-height: 16.39px;
  }
  .dlg-actions { height: 26px; display: flex; justify-content: center; align-items: center; gap: 10px; }
  .dlg-btn {
    height: 26px; padding: 0 8px; background: transparent; border: 1px solid;
    font-family: 'JetBrains Mono Bold'; font-size: 11px; line-height: 14.52px;
  }
  .dlg-btn.apply { border-color: var(--green); color: var(--green); }
  .dlg-btn.apply:hover { background: var(--greenTr); }
  .dlg-btn.deny { border-color: var(--red); color: var(--red); }
  .dlg-btn.deny:hover { background: var(--redTr); }
  .dlg-sep { width: 6px; height: 26px; display: flex; justify-content: center; }
  .dlg-sep::before { content: ''; width: 1px; height: 100%; background: var(--text-secondary); }
</style>
