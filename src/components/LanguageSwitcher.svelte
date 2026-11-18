<script>
  // Переключатель языка в шапке меню серверов. Образец: storefront LanguageSwitcher.tsx и renderLangSwitcher сайта (флаги UA/EN/RU).
  // Кнопка: флаг текущего языка 36x24 и шеврон. Клик открывает тёмную панель с флагами столбиком (UA, EN, RU).
  // Выбранный язык отмечен золотым кружком с галочкой. Клик вне панели и Esc закрывают её.
  // Язык be или pl: в списке только ru, en, uk, поэтому кнопка показывает нейтральный значок глобуса,
  // а в списке ничего не отмечено. Выбор любого флага вызывает общую setLocale (как в настройках).
  import { app, setLocale } from '../lib/app.svelte.js';
  import { LOCALES } from '../lib/i18n.svelte.js';
  import { tooltip } from '../lib/tooltip.js';

  // порядок как в дизайне сайта; code - код флага (ru/en/uk)
  const ITEMS = [
    { id: 'UKRAINIAN', code: 'uk' },
    { id: 'ENGLISH', code: 'en' },
    { id: 'RUSSIAN', code: 'ru' },
  ];

  let open = $state(false);
  let root = $state(null);

  // то же чтение, что у комбо в настройках: app.settings.locale, по умолчанию русский
  const cur = $derived(app.settings?.locale || 'RUSSIAN');
  const nameOf = id => (LOCALES.find(l => l.id === id) || LOCALES[0]).name;
  const curName = $derived(nameOf(cur));
  const curItem = $derived(ITEMS.find(i => i.id === cur));

  function pick(id) {
    open = false;
    setLocale(id);
  }
  function onPointer(e) {
    if (open && root && !root.contains(e.target)) open = false;
  }
  function onKey(e) {
    if (e.key === 'Escape') open = false;
  }
</script>

<svelte:window onpointerdown={onPointer} onkeydown={onKey} />

{#snippet flag(code)}
  {#if code === 'uk'}
    <svg viewBox="0 0 3 2" preserveAspectRatio="none" width="100%" height="100%" aria-hidden="true"><rect width="3" height="1" fill="#0057B7" /><rect width="3" height="1" y="1" fill="#FFD700" /></svg>
  {:else if code === 'en'}
    <svg viewBox="0 0 60 40" preserveAspectRatio="none" width="100%" height="100%" aria-hidden="true">
      <rect width="60" height="40" fill="#012169" /><path d="M0,0 L60,40" stroke="#fff" stroke-width="8" /><path d="M60,0 L0,40" stroke="#fff" stroke-width="8" /><path d="M0,0 L60,40" stroke="#C8102E" stroke-width="3" /><path d="M60,0 L0,40" stroke="#C8102E" stroke-width="3" /><rect
        x="24"
        width="12"
        height="40"
        fill="#fff"
      /><rect y="14" width="60" height="12" fill="#fff" /><rect x="26" width="8" height="40" fill="#C8102E" /><rect y="16" width="60" height="8" fill="#C8102E" />
    </svg>
  {:else}
    <svg viewBox="0 0 3 2" preserveAspectRatio="none" width="100%" height="100%" aria-hidden="true"><rect width="3" height="2" fill="#fff" /><rect width="3" height="1.333" y="0.667" fill="#0039A6" /><rect width="3" height="0.667" y="1.333" fill="#D52B1E" /></svg>
  {/if}
{/snippet}

<div class="sm-lang" bind:this={root} data-no-drag>
  <button type="button" class="sm-langBtn" aria-label={curName} aria-expanded={open} use:tooltip={curName} onclick={() => (open = !open)}>
    <span class="sm-langFlag">
      {#if curItem}
        {@render flag(curItem.code)}
      {:else}
        <svg class="sm-langGlobe" viewBox="0 0 24 24" width="24" height="24" aria-hidden="true">
          <circle cx="12" cy="12" r="10.5" fill="none" stroke="currentColor" stroke-width="1.6" /><ellipse cx="12" cy="12" rx="4.5" ry="10.5" fill="none" stroke="currentColor" stroke-width="1.4" /><path d="M1.5 12h21M3.5 7h17M3.5 17h17" fill="none" stroke="currentColor" stroke-width="1.4" />
        </svg>
      {/if}
    </span>
    <svg class="sm-langChev" class:up={open} viewBox="0 0 16 16" width="16" height="16" aria-hidden="true"><path d="M4 6l4 4 4-4" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" /></svg>
  </button>

  {#if open}
    <div class="sm-langPanel" role="listbox" aria-label={curName}>
      {#each ITEMS as it (it.id)}
        <button type="button" class="sm-langItem" role="option" aria-selected={it.id === cur} aria-label={nameOf(it.id)} use:tooltip={nameOf(it.id)} onclick={() => pick(it.id)}>
          <span class="sm-langFlagSm">{@render flag(it.code)}</span>
          {#if it.id === cur}
            <span class="sm-langCheck"><svg viewBox="0 0 16 16" width="10" height="10" aria-hidden="true"><path d="M3 8.5l3 3 7-7" fill="none" stroke="#120f00" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" /></svg></span>
          {/if}
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .sm-lang {
    position: relative;
    flex-shrink: 0;
  }
  .sm-langBtn {
    height: 46px;
    padding: 0 4px;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 12px;
    background: transparent;
    border: 0;
    cursor: pointer;
  }
  .sm-langBtn:hover {
    opacity: 0.85;
  }
  .sm-langFlag {
    width: 36px;
    height: 24px;
    border-radius: 3px;
    overflow: hidden;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    color: rgba(255, 255, 255, 0.75);
  }
  .sm-langFlag :global(svg) {
    display: block;
  }
  .sm-langGlobe {
    display: block;
  }
  .sm-langChev {
    display: block;
    color: rgba(255, 255, 255, 0.75);
    transition: transform 0.15s;
  }
  .sm-langChev.up {
    transform: rotate(180deg);
  }

  /* панель: 58 px, скругление 14, рамка 25 % белого, тень; позиция - под кнопкой, над телом шапки */
  .sm-langPanel {
    position: absolute;
    top: 50px;
    left: 50%;
    transform: translateX(-50%);
    z-index: 60;
    width: 58px;
    box-sizing: border-box;
    padding: 10px;
    border-radius: 14px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    background: var(--panel);
    border: 1px solid rgba(255, 255, 255, 0.25);
    box-shadow: 0 16px 40px rgba(0, 0, 0, 0.35);
  }
  .sm-langItem {
    position: relative;
    display: block;
    width: 36px;
    height: 24px;
    padding: 0;
    background: none;
    border: 0;
    cursor: pointer;
  }
  .sm-langItem:hover {
    filter: brightness(1.1);
  }
  .sm-langFlagSm {
    display: block;
    width: 36px;
    height: 24px;
    border-radius: 4px;
    overflow: hidden;
  }
  .sm-langFlagSm :global(svg) {
    display: block;
  }
  .sm-langCheck {
    position: absolute;
    top: -6px;
    right: -6px;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: var(--yellow);
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .sm-langCheck svg {
    display: block;
  }
</style>
