<script>
  // CheckBox из login.css/global.css: квадрат 22px (рамка 1 + отступ 4 + метка 12), метка-крестик (-fx-shape)
  let { checked = $bindable(false), label = '', onchange = null, class: cls = '' } = $props();
  function toggle() {
    checked = !checked;
    onchange && onchange(checked);
  }
</script>

<label class="check {cls}" class:on={checked} data-no-drag>
  <button type="button" class="box" role="checkbox" aria-checked={checked} onclick={toggle}>
    {#if checked}
      <svg class="mark" viewBox="10.9531 10.4531 11.0938 11.0938" preserveAspectRatio="none" aria-hidden="true">
        <path d="M11.7612 10.4531L10.9531 11.2612L15.692 16L10.9531 20.7388L11.7612 21.5469L16.5 16.808L21.2388 21.5469L22.0469 20.7388L17.308 16L22.0469 11.2612L21.2388 10.4531L16.5 15.192L11.7612 10.4531Z" />
      </svg>
    {/if}
  </button>
  {#if label}<span class="text" onclick={toggle} role="presentation">{label}</span>{/if}
</label>

<style>
  .check { display: inline-flex; align-items: center; gap: 8px; cursor: pointer; }
  .box {
    width: 22px; height: 22px; flex: none; padding: 4px;
    display: grid; place-items: center;
    background: var(--panel-flat);
    border: 1px solid var(--border-dark);
  }
  .on .box { border-color: var(--red); }
  .mark { width: 12px; height: 12px; fill: var(--red); }
  .text {
    font-family: 'JetBrains Mono'; font-size: 12px;
    color: var(--text-secondary); white-space: nowrap;
  }
  .on .text { color: var(--header-hover); }
</style>
