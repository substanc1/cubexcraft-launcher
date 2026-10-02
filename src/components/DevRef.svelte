<script>
  // Только для разработки (vite dev): наложение эталонного снимка JavaFX-экрана поверх новой вёрстки.
  // ?ref=scenes_login_login  (файл из app/dev-ref/), клавиша R: выкл -> разность -> 50% -> только эталон
  const params = new URLSearchParams(location.search);
  const ref = params.get('ref');
  const modes = ['off', 'diff', 'half', 'ref'];
  let mode = $state(params.get('mode') || 'diff');
  function key(e) {
    if (e.key === 'r' && !(e.target instanceof HTMLInputElement)) mode = modes[(modes.indexOf(mode) + 1) % modes.length];
  }
</script>

<svelte:window onkeydown={key} />
{#if ref && mode !== 'off'}
  <img class="ref {mode}" src="/dev-ref/{ref}.png" alt="" />
  <div class="badge">ref: {ref} · {mode} (R)</div>
{/if}

<style>
  .ref { position: fixed; left: 0; top: 0; pointer-events: none; z-index: 9999; }
  .ref.diff { mix-blend-mode: difference; }
  .ref.half { opacity: 0.5; }
  .badge { position: fixed; right: 4px; bottom: 4px; z-index: 10000; font: 10px monospace; color: #0f0; background: #000a; padding: 2px 4px; pointer-events: none; }
</style>
