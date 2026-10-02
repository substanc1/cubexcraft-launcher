<script>
  // scenes/servermenu/servermenu.fxml + ServerMenuScene + components/userBlock.fxml/UserBlock
  import './servermenu.css';
  import { onMount } from 'svelte';
  import { app, t, switchScene, processRequest, showApplyDialog, saveSettings, startDrag, errorHandle } from '../lib/app.svelte.js';
  import { call } from '../lib/backend.js';
  import { play } from '../lib/play.js';
  import IconButton from '../components/IconButton.svelte';
  import CutButton from '../components/CutButton.svelte';
  import Avatar from '../components/Avatar.svelte';
  import AccountMenu from '../components/AccountMenu.svelte';
  import { tooltip } from '../lib/tooltip.js';
  import { fxscroll } from '../lib/fxscroll.js';
  import logo from '../assets/images/logo.png';

  const MODS_COLLAPSED_LIMIT = 14;

  let nav = $state('servers');
  let navLabel = $state(t('scenes.servermenu.nav.servers'));
  let search = $state('');
  let filter = $state('ALL');
  let modsExpanded = $state(false);
  let pings = $state({});
  let menuAnchor = $state(null);
  let mainScroll;
  let sideScroll;
  let friendsSection;

  const prop = (p, k) => (p && p.properties ? p.properties[k] : undefined);
  const name = p => p.title || p.name || '';
  const maint = p => String(prop(p, 'maintenance')).toLowerCase() === 'true';

  const profiles = $derived([...app.profiles].sort((a, b) => name(a).localeCompare(name(b))));
  const categories = $derived([...new Set(profiles.map(p => prop(p, 'category')).filter(c => c && c.trim()))]);
  const filtered = $derived(
    profiles.filter(p => {
      const c = prop(p, 'category');
      if (filter !== 'ALL' && (!c || c.toLowerCase() !== filter.toLowerCase())) return false;
      const q = search.trim().toLowerCase();
      return !q || (name(p) + ' ' + p.version).toLowerCase().includes(q);
    }),
  );
  const selected = $derived(app.selected ? profiles.find(p => p.uuid === app.selected) || profiles[0] : profiles[0]);
  const mods = $derived(
    String(prop(selected, 'mods') || '')
      .split(',')
      .map(s => s.trim())
      .filter(Boolean),
  );
  const shownMods = $derived(modsExpanded || mods.length <= MODS_COLLAPSED_LIMIT ? mods : mods.slice(0, MODS_COLLAPSED_LIMIT));
  const selPing = $derived(selected ? pings[selected.uuid] : null);
  const disabled = $derived(!selected || maint(selected));

  function selectProfile(p) {
    app.selected = p.uuid;
    app.settings.lastProfile = p.uuid;
    modsExpanded = false;
    saveSettings();
  }

  onMount(() => {
    // ServerMenuScene.reset: пинг каждого сервера, выбранный - lastProfile или первый
    if (!app.selected) app.selected = app.settings?.lastProfile || null;
    for (const p of profiles) {
      const s = p.servers && p.servers[0];
      if (!s) continue;
      call('server_ping', { host: s.serverAddress, port: s.serverPort })
        .then(r => (pings[p.uuid] = r))
        .catch(() => {});
    }
  });

  function navTo(which) {
    nav = which;
    if (which === 'servers') {
      mainScroll.scrollTop = 0;
      navLabel = t('scenes.servermenu.nav.servers');
    } else if (which === 'friends') {
      sideScroll.scrollTop = friendsSection.offsetTop - sideScroll.offsetTop;
      navLabel = t('scenes.servermenu.nav.friends');
    } else if (which === 'news') {
      sideScroll.scrollTop = 0;
      navLabel = t('scenes.servermenu.nav.news');
    }
  }

  function doDeauth() {
    showApplyDialog(t('scenes.settings.exitDialog.header'), t('scenes.settings.exitDialog.description'), async () => {
      try {
        app.quickAccounts = await processRequest(t('scenes.settings.exitDialog.processing'), call('auth_logout'));
        app.user = null;
        app.settings.login = null;
        app.settings.hasSavedSession = false;
        await switchScene('login');
      } catch (e) {
        errorHandle(e);
      }
    });
  }

  function openConsole() {
    switchScene('debug');
  }
</script>

<div class="sm-root bg-image" onmousedown={startDrag} role="presentation">
  <!-- Левая иконочная панель -->
  <div class="sm-rail">
    <img class="sm-railLogo" src={logo} alt="" />
    <div class="sm-railDivider"></div>
    <button type="button" class="sm-railNav" class:selected={nav === 'servers'} use:tooltip={t('scenes.servermenu.nav.servers')} onclick={() => navTo('servers')}>▦</button>
    <button type="button" class="sm-railNav" class:selected={nav === 'friends'} use:tooltip={t('scenes.servermenu.nav.friends')} onclick={() => navTo('friends')}>◉</button>
    <button type="button" class="sm-railNav" class:selected={nav === 'news'} use:tooltip={t('scenes.servermenu.nav.news')} onclick={() => navTo('news')}>▤</button>
    <div class="sm-grow"></div>
    <button type="button" class="sm-railNav" use:tooltip={t('scenes.servermenu.nav.settings')} onclick={() => switchScene('settings')}>⚙</button>
    <button type="button" class="sm-deauth" use:tooltip={t('scenes.servermenu.deauthTooltip')} onclick={doDeauth}>⏻</button>
  </div>

  <!-- Шапка -->
  <div class="sm-header">
    <div class="sm-headerLeft">
      <span class="sm-word"><span>CUBE</span><span class="sm-accent">X</span><span>CRAFT</span></span>
      <span class="sm-navChip">{navLabel}</span>
      <label class="sm-searchBox" data-no-drag>
        <input class="sm-search" bind:value={search} placeholder={t('scenes.servermenu.search')} spellcheck="false" />
        <span class="sm-searchGlyph">⌕</span>
      </label>
    </div>
    <div class="sm-userBlock">
      <button type="button" class="sm-userCard" onclick={e => (menuAnchor = e.currentTarget.getBoundingClientRect())} data-no-drag>
        <span class="sm-avatarSlot"><Avatar username={app.user?.username} /></span>
        <span class="sm-nickname">{app.user?.username || 'Player'}</span>
        <span class="sm-switch" use:tooltip={t('scenes.servermenu.switchAccountTooltip')}>▾</span>
      </button>
    </div>
    <div class="sm-controls">
      <IconButton glyph="—" tip={t('scenes.servermenu.minimizeTooltip')} onclick={() => call('window_minimize')} />
      <IconButton glyph="✕" kind="exit" tip={t('scenes.servermenu.exitTooltip')} onclick={() => call('window_close')} />
    </div>
  </div>

  <div class="sm-body">
    <!-- Основная колонка -->
    <div class="sm-main scroll" bind:this={mainScroll} use:fxscroll={{ right: 0, top: 109, bottom: 109, width: 12 }}>
      <div class="sm-mainColumn">
        {#if selected}
          <div class="sm-hero sm-panel">
            <div class="sm-heroTagRow">
              <span class="sm-tag">{t('scenes.servermenu.heroTag')}</span>
              {#if prop(selected, 'wipeDate')}
                <span class="sm-badgeWipe">{t('scenes.servermenu.wipe')} {prop(selected, 'wipeDate')}</span>
              {/if}
            </div>
            <div class="sm-heroName">{name(selected)}</div>
            <div class="sm-heroChips">
              <span class="sm-heroChip">{selected.version}</span>
              {#if prop(selected, 'mode')}<span class="sm-heroChip">{prop(selected, 'mode')}</span>{/if}
            </div>
            <div class="sm-heroDesc"><span>{selected.info || ''}</span></div>
            <div class="sm-onlineBox">
              <div class="sm-onlineRow">
                <span class="sm-onlineValue">{selPing ? selPing.online : '—'}</span>
                <span class="sm-onlineMax">{selPing ? '/ ' + selPing.max : ''}</span>
              </div>
              <div class="sm-onlineCaption">{t('scenes.servermenu.onlineCaption')}</div>
              <div class="sm-loadTrack">
                <div class="sm-loadFill" style="width:{selPing && selPing.max > 0 ? 200 * Math.min(1, selPing.online / selPing.max) : 0}px"></div>
              </div>
            </div>
          </div>
        {/if}

        <div class="sm-gridSection">
          <div class="sm-gridHead">
            <span class="sm-tag">{t('scenes.servermenu.gridTag')}</span>
            <span class="sm-grow"></span>
            {#if categories.length}
              <div class="sm-filters">
                <button type="button" class="sm-filterChip" class:active={filter === 'ALL'} onclick={() => (filter = 'ALL')}>{t('scenes.servermenu.filterAll')}</button>
                {#each categories as c}
                  <button type="button" class="sm-filterChip" class:active={filter === c} onclick={() => (filter = c)}>{c}</button>
                {/each}
              </div>
            {/if}
          </div>
          <div class="sm-grid">
            {#each filtered as p (p.uuid)}
              {@const sel = selected && p.uuid === selected.uuid}
              {@const m = maint(p)}
              {@const pg = pings[p.uuid]}
              <button type="button" class="sm-card" class:active={sel} class:maintenance={m} onclick={() => selectProfile(p)} data-no-drag>
                <span class="sm-cardRail" class:active={sel} class:maintenance={m && !sel}></span>
                <span class="sm-cardContent">
                  <span class="sm-cardTop">
                    <span class="sm-cardName">{name(p)}</span>
                    <span class="sm-dot" class:maint={m}></span>
                  </span>
                  <span class="sm-cardMeta">{p.version}{prop(p, 'mode') ? ' · ' + prop(p, 'mode') : ''}</span>
                  <span class="sm-cardTrack">
                    <span class="sm-cardFill" class:active={sel} style="width:{pg && pg.max > 0 && !m ? 100 * Math.min(1, pg.online / pg.max) : 0}%"></span>
                  </span>
                  <span class="sm-cardBottom">
                    <span class="sm-cardOnline">{m ? t('scenes.servermenu.unavailable') : pg ? pg.online + '/' + pg.max : '?'}</span>
                    <span class="sm-grow"></span>
                    <span class="sm-cardTag" class:active={sel}>{(prop(p, 'category') || '').toUpperCase()}</span>
                  </span>
                </span>
              </button>
            {/each}
          </div>
        </div>

        {#if selected && mods.length}
          <div class="sm-mods sm-panel">
            <div class="sm-modsHead">
              <span class="sm-tag">{t('scenes.servermenu.modsTag')}</span>
              <span class="sm-modsCount">{mods.length} {t('scenes.servermenu.mods')}</span>
            </div>
            <div class="sm-modsList">
              {#each shownMods as mod}<span class="sm-modChip">{mod}</span>{/each}
              {#if mods.length > MODS_COLLAPSED_LIMIT}
                <button type="button" class="sm-modChip sm-modToggle" onclick={() => (modsExpanded = !modsExpanded)}>
                  {modsExpanded ? t('scenes.servermenu.modsCollapse') : t('scenes.servermenu.modsExpand')}
                </button>
              {/if}
            </div>
          </div>
        {/if}
      </div>
    </div>

    <!-- Правая колонка -->
    <div class="sm-side scroll" bind:this={sideScroll} use:fxscroll={{ right: 5, top: 13, bottom: 13, width: 10 }}>
      <div class="sm-sideColumn">
        <div class="sm-sideSection">
          <span class="sm-tag">{t('scenes.servermenu.feedTag')}</span>
          <span class="sm-emptyState">{t('scenes.servermenu.comingSoon')}</span>
        </div>
        <div class="sm-sideSection" bind:this={friendsSection}>
          <span class="sm-tag">{t('scenes.servermenu.friendsTag')}</span>
          <span class="sm-emptyState">{t('scenes.servermenu.comingSoon')}</span>
        </div>
      </div>
    </div>
  </div>

  <!-- Док запуска -->
  <div class="sm-dock">
    <div class="sm-dockRow">
      <div class="sm-dockTitle">
        <span class="sm-dockCaption">{t('scenes.servermenu.dockCaption')}</span>
        <span class="sm-dockName">{selected ? name(selected) : ''}</span>
      </div>
      <div class="sm-dockChips">
        {#if selected}
          <span class="sm-dockChip">{selected.version}</span>
          <span class="sm-dockChip">Java {app.javaVersion}</span>
          <button type="button" class="sm-dockChip sm-dockConsole" onclick={openConsole}>{t('scenes.servermenu.consoleChip', 'КОНСОЛЬ')}</button>
        {/if}
      </div>
      <div class="sm-dockStatusBox">
        <div class="sm-dockStatusRow">
          <span class="sm-dockStatus">{selected && maint(selected) ? t('scenes.servermenu.unavailable') : t('scenes.servermenu.dockReady')}</span>
          <span class="sm-grow"></span>
          <span class="sm-dockPercent">0%</span>
        </div>
        <div class="sm-dockTrack"><div class="sm-dockFill" style="width:0"></div></div>
      </div>
    </div>
    <CutButton class="sm-play" variant={disabled ? 'disabled' : 'play'} shape="M0,0 L168,0 L180,12 L180,44 L12,44 L0,32 Z" viewBox="0 0 180 44"
               disabled={disabled} onclick={() => selected && play(selected)}>
      {t('scenes.servermenu.play')}
    </CutButton>
  </div>
</div>

{#if menuAnchor}
  <AccountMenu anchor={menuAnchor} onclose={() => (menuAnchor = null)} />
{/if}
