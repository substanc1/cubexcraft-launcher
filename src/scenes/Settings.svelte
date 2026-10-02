<script>
  // scenes/settings/globalsettings.fxml + GlobalSettingsScene/BaseSettingsScene/LanguageSelector
  import './settings.css';
  import { app, t, back, saveSettings, startDrag, errorHandle, notify } from '../lib/app.svelte.js';
  import { i18n, LOCALES } from '../lib/i18n.svelte.js';
  import { call } from '../lib/backend.js';
  import { applyTheme, hexToRgb, hsb, fitAccent, fitBackground, fitText, fitLines, DEFAULTS } from '../lib/theme.js';
  import IconButton from '../components/IconButton.svelte';
  import { tooltip } from '../lib/tooltip.js';
  import { fxscroll } from '../lib/fxscroll.js';

  const s = app.settings;
  const OPTIONS = [
    ['prismvsync', 'prismVSync'],
    ['debugallclients', 'debugAllClients'],
    ['reduceeffects', 'reduceEffects'],
  ];
  const RAM = [4, 6, 8, 12];
  const COLORS = [
    ['accent', fitAccent],
    ['background', fitBackground],
    ['text', fitText],
    ['lines', fitLines],
  ];

  // ОЗУ - для текущего профиля (или первого), как addRamSectionUnsafe
  const ramProfile = $derived(app.selected || (app.profiles[0] && app.profiles[0].uuid));
  const ram = $derived((ramProfile && s.profiles && s.profiles[ramProfile] && s.profiles[ramProfile].ram) || 0);
  // ширина как у JavaFX: моноширинный 11px (шаг 6.6) + отступы 32 + рамка 2, округление вверх
  const ramWidth = label => Math.ceil(label.length * 6.6 + 34);
  let openPicker = $state(null);
  let hex = $state('');
  let langOpen = $state(false);

  function toggle(key) {
    s[key] = !s[key];
    saveSettings();
  }
  function setRam(gb) {
    if (!ramProfile) return;
    s.profiles = s.profiles || {};
    s.profiles[ramProfile] = { ...(s.profiles[ramProfile] || {}), ram: gb * 1024 };
    saveSettings();
  }
  function setColor(key, value) {
    s.customColors = { ...(s.customColors || {}), [key]: value };
    applyTheme(s);
    saveSettings();
  }
  function resetColors() {
    s.customColors = {};
    applyTheme(s);
    saveSettings();
  }
  // палитра: 16 оттенков по кругу, подогнанных под роль цвета (buildColorPalettePopup)
  function palette(fit) {
    return Array.from({ length: 16 }, (_, i) => fit(hsb(i * 22.5, 1, 1)));
  }
  function applyHex(key, fit) {
    const rgb = hexToRgb(hex.startsWith('#') ? hex : '#' + hex);
    if (!rgb) return;
    setColor(key, fit(rgb));
    openPicker = null;
  }
  function setLocale(id) {
    langOpen = false;
    if (id === s.locale) return;
    s.locale = id;
    i18n.locale = id;
    saveSettings();
  }

  async function changeDir() {
    try {
      const dir = await call('updates_dir_pick');
      if (!dir) return;
      const r = await call('updates_dir_change', { path: dir });
      s.updatesDir = r;
    } catch (e) {
      errorHandle(e);
    }
  }
</script>

<svelte:window onmousedown={e => { if (!e.target.closest('.st-picker, .st-swatchBtn')) openPicker = null; if (!e.target.closest('.st-combo')) langOpen = false; }} />

<div class="st-layout bg-image" onmousedown={startDrag} role="presentation">
  <div class="st-header">
    <span class="st-back"><IconButton glyph="←" tip={t('scenes.serverinfo.backTooltip')} onclick={back} /></span>
    <span class="st-title">{t('scenes.settings.header')}</span>
    <div class="st-controls">
      <IconButton glyph="—" tip={t('scenes.servermenu.minimizeTooltip')} onclick={() => call('window_minimize')} />
      <IconButton glyph="✕" kind="exit" tip={t('scenes.servermenu.exitTooltip')} onclick={() => call('window_close')} />
    </div>
  </div>

  <div class="st-list">
   <div class="st-viewport scroll" use:fxscroll={{ right: -43, top: 94, bottom: 95, width: 12 }}>
    <div class="st-content">
      <div class="st-card st-settingsList">
        <span class="st-h">{t('scenes.settings.header.options')}</span>
        {#each OPTIONS as [lang, key]}
          <div class="st-row">
            <button type="button" class="st-check" class:on={s[key]} onclick={() => toggle(key)} aria-label={lang}>
              {#if s[key]}
                <svg viewBox="-0.3 -1.1 16.4 17.3" preserveAspectRatio="none"><path d="M-0.25,6.083c0.843-0.758,4.583,4.833,5.75,4.833S14.5-1.5,15.917-0.917c1.292,0.532-8.75,17.083-10.5,17.083C3,16.167-1.083,6.833-0.25,6.083z" /></svg>
              {/if}
            </button>
            <div class="st-texts">
              <span class="st-lh">{t(`scenes.settings.properties.${lang}.name`, lang)}</span>
              <span class="st-ld">{t(`scenes.settings.properties.${lang}.description`, lang)}</span>
            </div>
          </div>
        {/each}

        {#if ramProfile}
          <span class="st-h">{t('scenes.settings.header.ram')}</span>
          <div class="st-ramRow">
            {#each RAM as gb}
              <button type="button" class="st-ram" style="width:{ramWidth(gb + ' ГБ')}px" class:selected={ram === gb * 1024} onclick={() => setRam(gb)}>{gb} ГБ</button>
            {/each}
            <button type="button" class="st-ram" style="width:{ramWidth(t('scenes.settings.ramAuto'))}px" class:selected={ram === 0} onclick={() => setRam(0)}>{t('scenes.settings.ramAuto')}</button>
          </div>
        {/if}

        <span class="st-h">{t('scenes.settings.header.colors', 'Цвета интерфейса')}</span>
        {#each COLORS as [key, fit]}
          {@const value = (s.customColors && s.customColors[key]) || DEFAULTS[key]}
          <div class="st-row">
            <div class="st-swatchWrap">
              <button type="button" class="st-swatchBtn" onclick={() => { openPicker = openPicker === key ? null : key; hex = ''; }}>
                <span class="st-swatch" style="background:{value}"></span>
                <span class="st-arrow">▾</span>
              </button>
              {#if openPicker === key}
                <div class="st-picker">
                  <div class="st-grid">
                    {#each palette(fit) as c}
                      <button type="button" class="st-cell" style="background:{c}" aria-label={c} onclick={() => { setColor(key, c); openPicker = null; }}></button>
                    {/each}
                  </div>
                  <input class="st-hex" placeholder="#RRGGBB" bind:value={hex} onkeydown={e => e.key === 'Enter' && applyHex(key, fit)} />
                </div>
              {/if}
            </div>
            <div class="st-texts">
              <span class="st-lh">{t(`scenes.settings.properties.color.${key}.name`, key)}</span>
              <span class="st-ld">{t(`scenes.settings.properties.color.${key}.description`, '')}</span>
            </div>
          </div>
        {/each}
        <div class="st-row">
          <button type="button" class="st-action" onclick={resetColors}>{t('scenes.settings.properties.resetcolors.name', 'Сбросить цвета')}</button>
        </div>
      </div>

      <div class="st-lang">
        <span class="st-h">{t('scenes.settings.header.language')}</span>
        <div class="st-langRow">
          <div class="st-combo" use:tooltip={(LOCALES.find(l => l.id === s.locale) || LOCALES[0]).name}>
            <button type="button" class="st-comboBtn" onclick={() => (langOpen = !langOpen)}>
              <span>{(LOCALES.find(l => l.id === (s.locale || 'RUSSIAN')) || LOCALES[0]).name}</span>
            </button>
            {#if langOpen}
              <div class="st-comboList">
                {#each LOCALES as l}
                  <button type="button" class="st-comboItem" class:selected={l.id === (s.locale || 'RUSSIAN')} onclick={() => setLocale(l.id)}>{l.name}</button>
                {/each}
              </div>
            {/if}
          </div>
          <span class="st-ld st-langDesc">{t('scenes.settings.languageDesc')}</span>
        </div>
      </div>

      <div class="st-card">
        <span class="st-h">{t('scenes.settings.header.changeDirectory')}</span>
        <div class="st-folder">
          <button type="button" class="st-path" use:tooltip={s.updatesDir || app.updatesDir} onclick={() => call('open_path', { path: s.updatesDir || app.updatesDir })}>{s.updatesDir || app.updatesDir}</button>
          <button type="button" class="icon-button st-changeDir" use:tooltip={t('scenes.settings.changedirTooltip')} onclick={changeDir} aria-label={t('scenes.settings.dirText')}>
            <svg viewBox="0 0 24 24" width="24" height="24"><path d="M12 6.3999H6.4H5V7.7999V16.1999V17.5999H19V16.1999L19 9.1999V7.7999H17.6H12V6.3999ZM10.6 9.1999H17.6V16.1999H6.4V7.7999H10.6V9.1999ZM14.7998 11.9999H13.4V10.5999H12V11.9999H13.3998V13.3999H14.7998V11.9999ZM14.7998 11.9999H16.1998V10.5999H14.7998L14.7998 11.9999ZM16.1998 13.3999H14.7998V14.7999H16.1998V13.3999ZM13.4 13.3999H12V14.7999H13.4V13.3999Z" /></svg>
          </button>
        </div>
      </div>
    </div>
   </div>
  </div>
</div>
