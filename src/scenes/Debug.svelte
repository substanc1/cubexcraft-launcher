<script>
  // Журнал игры (бывший debug.fxml): записи по уровням - вкладки ВСЕ / ОШИБКИ / ПРЕДУПРЕЖДЕНИЯ / ИНФО / ОТЛАДКА /
  // ПРОЧЕЕ со счётчиками, поиск, стек ошибки сворачивается, повторы - «×N». Сверху статус игры; при вылете ядро
  // сохраняет журнал в cxl/logs/crashes - ссылка на папку. Копировать - показанные записи (с учётом вкладки и поиска).
  // Закрытие при запущенной игре - подтверждение (forceExitDialog).
  import { app, t, back, startDrag, showApplyDialog } from '../lib/app.svelte.js';
  import { call } from '../lib/backend.js';
  import IconButton from '../components/IconButton.svelte';
  import { fxscroll } from '../lib/fxscroll.js';
  import { log, entries, entryText } from '../lib/gamelog.svelte.js';

  const TABS = [
    ['all', 'scenes.debug.tabAll'],
    ['error', 'scenes.debug.tabError'],
    ['warn', 'scenes.debug.tabWarn'],
    ['info', 'scenes.debug.tabInfo'],
    ['debug', 'scenes.debug.tabDebug'],
    ['other', 'scenes.debug.tabOther'],
  ];
  const STEP = 500; // записей в окне за раз (больше - кнопка «раньше»)

  let tab = $state('all');
  let query = $state('');
  let limit = $state(STEP);
  let open = $state({}); // id записи -> стек раскрыт
  let follow = $state(true); // держаться у конца журнала
  let out;

  const filtered = $derived.by(() => {
    log.rev;
    const q = query.trim().toLowerCase();
    let list = tab === 'all' ? entries.slice() : entries.filter(e => e.level === tab);
    if (q) list = list.filter(e => (e.msg + '\n' + e.src + '\n' + e.thread).toLowerCase().includes(q) || e.more.some(l => l.toLowerCase().includes(q)));
    return list;
  });
  const shown = $derived(filtered.length > limit ? filtered.slice(filtered.length - limit) : filtered);
  const hidden = $derived(filtered.length - shown.length);

  // смена вкладки/поиска - снова к концу журнала
  $effect(() => {
    tab;
    query;
    limit = STEP;
    follow = true;
  });

  $effect(() => {
    shown;
    if (follow) queueMicrotask(() => out && (out.scrollTop = out.scrollHeight));
  });

  function onScroll() {
    follow = out.scrollHeight - out.scrollTop - out.clientHeight < 40;
  }
  function toEnd() {
    follow = true;
    out.scrollTop = out.scrollHeight;
  }
  function earlier() {
    const h = out.scrollHeight;
    limit += STEP;
    follow = false;
    queueMicrotask(() => (out.scrollTop += out.scrollHeight - h));
  }
  function toggle(id) {
    open[id] = !open[id];
  }

  // первая строка стека (сообщение исключения) видна сразу, остальное - по кнопке
  function headLine(e) {
    const l = e.more[0];
    return l && !/^\s/.test(l) && !l.startsWith('at ') ? l : '';
  }
  function traceRest(e) {
    return headLine(e) ? e.more.slice(1) : e.more;
  }

  function copy() {
    navigator.clipboard && navigator.clipboard.writeText(filtered.map(entryText).join('\n'));
  }
  function exit() {
    if (!app.gameRunning) return call('window_close');
    showApplyDialog(t('scenes.debug.forceExitDialog.header'), t('scenes.debug.forceExitDialog.description'), () => call('window_close'));
  }

  const status = $derived(
    app.gameRunning
      ? ['run', t('scenes.debug.running')]
      : log.exit
        ? log.exit.killed
          ? ['stop', t('scenes.debug.killed')]
          : log.exit.code === 0
            ? ['stop', t('scenes.debug.exited').replace('{code}', log.exit.code)]
            : ['crash', t('scenes.debug.crashed').replace('{code}', log.exit.code)]
        : null,
  );
</script>

<div class="dbg-layout bg-image" onmousedown={startDrag} role="presentation">
  <div class="dbg-header">
    <span class="dbg-back"><IconButton glyph="←" tip={t('scenes.debug.backTooltip')} onclick={back} /></span>
    <span class="dbg-title">{t('scenes.debug.headingDebug')}</span>
    <div class="dbg-meta">
      {#if status}
        <span class="dbg-status {status[0]}"><i></i>{status[1]}</span>
      {/if}
      {#if log.exit && log.exit.saved}
        <button type="button" class="dbg-saved" onclick={() => call('logs_open', { crash: true })}>
          {t('scenes.debug.saved').replace('{file}', log.exit.saved)} ↗
        </button>
      {/if}
    </div>
    <div class="dbg-controls">
      <IconButton glyph="⧉" tip={t('scenes.debug.copyTooltip')} onclick={copy} />
      <IconButton glyph="↗" tip={t('scenes.debug.openLogsTooltip')} onclick={() => call('logs_open', {})} />
      <IconButton glyph="■" kind="kill" tip={t('scenes.debug.killTooltip')} onclick={() => call('game_kill')} />
      <IconButton glyph="—" tip={t('scenes.servermenu.minimizeTooltip')} onclick={() => call('window_minimize')} />
      <IconButton glyph="✕" kind="exit" tip={t('scenes.servermenu.exitTooltip')} onclick={exit} />
    </div>
  </div>

  <div class="dbg-tools" data-no-drag>
    <div class="dbg-tabs">
      {#each TABS as [id, key]}
        {@const n = id === 'all' ? log.total : log.counts[id]}
        {#if id === 'all' || id === 'error' || id === 'warn' || id === 'info' || n}
          <button type="button" class="dbg-tab t-{id}" class:active={tab === id} onclick={() => (tab = id)}>
            {t(key)}<span class="dbg-count">{n}</span>
          </button>
        {/if}
      {/each}
    </div>
    <label class="dbg-search">
      <span class="dbg-searchIcon">⌕</span>
      <input bind:value={query} placeholder={t('scenes.debug.search')} spellcheck="false" />
      {#if query}<button type="button" class="dbg-clear" onclick={() => (query = '')}>✕</button>{/if}
    </label>
  </div>

  <div class="dbg-outputDetail">
    <div class="dbg-output scroll" bind:this={out} onscroll={onScroll} use:fxscroll={{ right: -17, top: 4, bottom: 4, width: 12 }}>
      {#if hidden > 0}
        <button type="button" class="dbg-earlier" onclick={earlier}>{t('scenes.debug.earlier').replace('{n}', hidden)}</button>
      {/if}
      {#each shown as e (e.id + ':' + e.n + ':' + e.more.length)}
        <div class="dbg-row lv-{e.level}">
          <span class="dbg-time">{e.time}</span>
          <span class="dbg-lvCell">{#if e.tag}<span class="dbg-lv">{e.tag}</span>{/if}</span>
          <span class="dbg-src" title={e.thread ? `${e.thread} · ${e.src}` : ''}>{e.short}</span>
          <span class="dbg-msg">
            <span class="dbg-text">{e.msg}</span>{#if e.n > 1}<span class="dbg-rep">×{e.n}</span>{/if}
            {#if e.more.length}
              {#if headLine(e)}<span class="dbg-exc">{headLine(e)}</span>{/if}
              {#if traceRest(e).length}
                <button type="button" class="dbg-traceBtn" onclick={() => toggle(e.id)}>
                  {open[e.id] ? '▾' : '▸'} {t('scenes.debug.trace').replace('{n}', traceRest(e).length)}
                </button>
                {#if open[e.id]}<span class="dbg-trace">{traceRest(e).join('\n')}</span>{/if}
              {/if}
            {/if}
          </span>
        </div>
      {:else}
        <div class="dbg-empty">{query ? t('scenes.debug.notFound') : t('scenes.debug.empty')}</div>
      {/each}
    </div>
    {#if !follow}
      <button type="button" class="dbg-toEnd" onclick={toEnd} data-no-drag>{t('scenes.debug.toEnd')}</button>
    {/if}
  </div>
</div>

<style>
  .dbg-layout { position: relative; width: 100%; height: 100%; overflow: hidden; }
  .dbg-header { position: absolute; left: 0; right: 0; top: 0; height: 100px; border: 1px solid transparent; border-bottom-color: var(--border-dark); }
  .dbg-back { position: absolute; left: 28px; top: 34px; }
  .dbg-title {
    position: absolute; left: 73px; top: 34px; height: 32px; display: flex; align-items: center;
    font-family: 'Unbounded SemiBold'; font-size: 18px; line-height: 22.32px; color: var(--text-hover); white-space: nowrap;
  }
  .dbg-meta { position: absolute; left: 290px; right: 300px; top: 34px; height: 32px; display: flex; align-items: center; gap: 14px; min-width: 0; }
  .dbg-status {
    height: 20px; flex: none; display: flex; align-items: center; gap: 8px; padding: 0 10px;
    font-family: 'JetBrains Mono'; font-size: 10.5px; letter-spacing: .14em; white-space: nowrap;
    border: 1px solid #3a0d13; color: var(--text-secondary);
  }
  .dbg-status i { width: 7px; height: 7px; background: currentColor; }
  .dbg-status.run { color: var(--green); border-color: #0f4a35; }
  .dbg-status.run i { box-shadow: 0 0 8px var(--green); animation: dbg-pulse 1.4s ease-in-out infinite; }
  .dbg-status.crash { color: var(--red-hover); border-color: var(--red); background: #ff2a3c18; }
  @keyframes dbg-pulse { 50% { opacity: .35; } }
  .dbg-saved {
    min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; padding: 0; border: 0; background: none; cursor: pointer;
    font-family: 'JetBrains Mono'; font-size: 10.5px; color: var(--text-secondary); text-decoration: underline dotted;
  }
  .dbg-saved:hover { color: var(--text-hover); }
  .dbg-controls { position: absolute; right: 29px; top: 34px; display: flex; gap: 10px; }

  /* вкладки уровней и поиск */
  .dbg-tools { position: absolute; left: 50px; right: 50px; top: 112px; height: 32px; display: flex; align-items: center; gap: 12px; }
  .dbg-tabs { display: flex; gap: 6px; flex: 1; min-width: 0; }
  .dbg-tab {
    display: flex; align-items: center; gap: 8px; height: 30px; padding: 0 12px; cursor: pointer; white-space: nowrap;
    font-family: 'JetBrains Mono'; font-size: 10.5px; letter-spacing: .14em;
    border: 1px solid #3a0d13; color: var(--text-secondary); background: transparent;
    transition: border-color .15s, color .15s, background .15s;
  }
  .dbg-tab:hover { color: var(--text-hover); border-color: var(--border-light); }
  .dbg-tab.active { border-color: var(--red); color: #fff; background: #ff2a3c22; }
  .dbg-count { color: var(--text-secondary); font-size: 10px; letter-spacing: .04em; }
  .dbg-tab.active .dbg-count { color: var(--header); }
  .dbg-tab.t-error .dbg-count { color: var(--red-hover); }
  .dbg-tab.t-warn .dbg-count { color: var(--yellow); }
  .dbg-search {
    width: 260px; height: 30px; display: flex; align-items: center; gap: 8px; padding: 0 10px; cursor: text;
    background: var(--panel-flat); box-shadow: inset 0 0 0 1px #3a0d13;
  }
  .dbg-search:focus-within { box-shadow: inset 0 0 0 1px var(--border-light); }
  .dbg-searchIcon { color: var(--text-secondary); font-size: 14px; }
  .dbg-search input {
    flex: 1; min-width: 0; border: 0; outline: 0; background: none; color: var(--text);
    font-family: 'JetBrains Mono'; font-size: 11.5px;
  }
  .dbg-search input::placeholder { color: var(--text-secondary); }
  .dbg-clear { border: 0; background: none; color: var(--text-secondary); cursor: pointer; font-size: 11px; padding: 0; }
  .dbg-clear:hover { color: var(--text-hover); }

  /* журнал */
  .dbg-outputDetail {
    position: absolute; left: 50px; right: 50px; top: 152px; bottom: 24px;
    background: var(--panel-flat); border: 1px solid var(--border-dark);
  }
  .dbg-output {
    position: absolute; inset: 0; overflow-y: auto; padding: 6px 0;
    user-select: text; -webkit-user-select: text; cursor: text;
    font-family: 'JetBrains Mono'; font-size: 12px; line-height: 17px; color: var(--text);
  }
  .dbg-row {
    display: grid; grid-template-columns: 64px 56px 150px minmax(0, 1fr); column-gap: 10px;
    padding: 2px 14px 2px 12px; border-left: 2px solid transparent;
  }
  .dbg-row:hover { background: #ffffff06; }
  .dbg-time { color: #6d4c51; }
  .dbg-lvCell { display: flex; align-items: flex-start; padding-top: 1px; }
  .dbg-lv {
    min-width: 46px; text-align: center; font-size: 9.5px; line-height: 14px; letter-spacing: .1em;
    border: 1px solid #3a0d13; color: var(--text-secondary);
  }
  .dbg-src { color: var(--text-secondary); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .dbg-msg { overflow-wrap: anywhere; color: #e8d6d8; }
  .dbg-text, .dbg-exc, .dbg-trace { white-space: pre-wrap; }
  .dbg-rep {
    margin-left: 8px; padding: 0 5px; font-size: 10px; color: var(--header);
    border: 1px solid #3a0d13; white-space: nowrap; user-select: none;
  }

  .lv-info .dbg-msg { color: #d8c4c7; }
  .lv-warn { border-left-color: var(--border-warn); }
  .lv-warn .dbg-lv { color: var(--yellow); border-color: var(--border-warn); background: #fcee0a10; }
  .lv-warn .dbg-msg { color: #efe5ae; }
  .lv-error { border-left-color: var(--red); background: #ff2a3c0d; }
  .lv-error .dbg-lv { color: #fff; border-color: var(--red); background: #ff2a3c55; }
  .lv-error .dbg-msg { color: #ffc9cd; }
  .lv-debug .dbg-lv { color: var(--violet-text); border-color: var(--violet-border); }
  .lv-debug .dbg-msg { color: #a98fb8; }
  .lv-other .dbg-msg { color: var(--header); }

  .dbg-exc { display: block; color: var(--red-hover); }
  .lv-warn .dbg-exc { color: #f5d76e; }
  .dbg-traceBtn {
    display: block; margin: 2px 0; padding: 0; border: 0; background: none; cursor: pointer;
    font-family: 'JetBrains Mono'; font-size: 11px; color: var(--text-secondary);
  }
  .dbg-traceBtn:hover { color: var(--text-hover); }
  .dbg-trace { display: block; color: #9c7d82; font-size: 11px; line-height: 15px; }

  .dbg-earlier {
    display: block; margin: 4px auto 8px; padding: 6px 14px; cursor: pointer;
    font-family: 'JetBrains Mono'; font-size: 10.5px; letter-spacing: .14em;
    border: 1px solid #3a0d13; color: var(--text-secondary); background: transparent;
  }
  .dbg-earlier:hover { color: var(--text-hover); border-color: var(--border-light); }
  .dbg-empty { padding: 40px 0; text-align: center; color: var(--text-secondary); font-size: 11px; letter-spacing: .14em; }
  .dbg-toEnd {
    position: absolute; right: 22px; bottom: 14px; padding: 7px 14px; cursor: pointer;
    font-family: 'JetBrains Mono'; font-size: 10.5px; letter-spacing: .14em;
    border: 1px solid var(--red); color: #fff; background: #1a0609ee; box-shadow: var(--glow-red);
  }
  .dbg-toEnd:hover { background: #2a0a10; }
</style>
