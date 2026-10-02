<script>
  // Окно «Друзья» (макет handoff-10, CubeXCraft Launcher.dc.html - friends popup): вкладки ДРУЗЬЯ / ЗАПРОСЫ /
  // УВЕДОМЛЕНИЯ, добавить по нику, пригласить на выбранную сборку, удалить, принять/отклонить, прочитать все.
  // Своя палитра «стальной» панели из макета (не тема лаунчера).
  import { t } from '../lib/app.svelte.js';
  import Avatar from './Avatar.svelte';
  import { popIn, popOut, fadeQuick } from '../lib/transitions.js';
  import {
    friends, closeFriends, sendRequest, acceptRequest, declineRequest, removeFriend, inviteFriend,
    readAllNotes, dismissNote, statusText, noteText, noteTime, requestNote,
  } from '../lib/friends.svelte.js';

  let { server = '' } = $props();
  let addVal = $state('');
  let now = $state(Date.now());

  $effect(() => {
    // длительность игры у друзей и «N мин назад» тикают, пока окно открыто
    const iv = setInterval(() => (now = Date.now()), 30000);
    return () => clearInterval(iv);
  });

  const onlineCount = $derived(friends.friends.filter(f => f.state !== 'offline').length);
  const unread = $derived(friends.notifications.filter(n => n.unread).length);
  const tabs = $derived([
    ['friends', t('friends.tabFriends'), friends.friends.length],
    ['reqs', t('friends.tabRequests'), friends.requests.length],
    ['notes', t('friends.tabNotes'), unread],
  ]);
  const list = $derived(friends.tab === 'friends' ? friends.friends : friends.tab === 'reqs' ? friends.requests : friends.notifications);

  async function send() {
    const v = addVal.trim();
    if (!v) return;
    if (await sendRequest(v)) addVal = '';
  }
  function pick(tab) {
    friends.tab = tab;
    friends.toast = '';
  }
</script>

{#if friends.open}
  <div class="fp-wrap" onclick={closeFriends} role="presentation" in:fadeQuick out:fadeQuick={{ duration: 160 }} data-no-drag>
    <div class="fp-panel" onclick={e => e.stopPropagation()} onkeydown={e => e.key === 'Escape' && closeFriends()} role="dialog" tabindex="-1" in:popIn out:popOut>
      <div class="fp-head">
        <span class="fp-title">{t('friends.title')}</span>
        <span class="fp-online">// {t('friends.online').replace('{n}', onlineCount + '/' + friends.friends.length)}</span>
        <span class="fp-grow"></span>
        <button type="button" class="fp-close" onclick={closeFriends}>✕</button>
      </div>
      <div class="fp-hazard"></div>
      <div class="fp-tabs">
        {#each tabs as [id, label, count]}
          <button type="button" class="fp-tab" class:active={friends.tab === id} onclick={() => pick(id)}>
            {label}{#if count}<span class="fp-tabCount">{count}</span>{/if}
          </button>
        {/each}
      </div>
      <div class="fp-addRow">
        <label class="fp-addBox">
          <span class="fp-plus">+</span>
          <input bind:value={addVal} placeholder={t('friends.add')} spellcheck="false" maxlength="32"
                 onkeydown={e => e.key === 'Enter' && send()} />
        </label>
        <button type="button" class="fp-send" onclick={send}>{t('friends.send')}</button>
      </div>

      <div class="fp-list">
        {#if friends.tab === 'friends'}
          {#each friends.friends as f (f.username)}
            <div class="fp-row">
              <span class="fp-avatar" class:off={f.state === 'offline'}><Avatar username={f.username} size={26} /></span>
              <div class="fp-who">
                <div class="fp-name">{f.username}</div>
                <div class="fp-sub">{statusText(f, now)}</div>
              </div>
              <span class="fp-dot" class:on={f.state === 'playing' || f.state === 'online'} class:away={f.state === 'away'}></span>
              <button type="button" class="fp-btn yellow" onclick={() => inviteFriend(f, server)}>{t('friends.invite')}</button>
              <button type="button" class="fp-btn red" onclick={() => removeFriend(f.username)}>{t('friends.remove')}</button>
            </div>
          {/each}
        {:else if friends.tab === 'reqs'}
          {#each friends.requests as r (r.username)}
            <div class="fp-row">
              <span class="fp-avatar"><Avatar username={r.username} size={26} /></span>
              <div class="fp-who">
                <div class="fp-name">{r.username}</div>
                <div class="fp-sub">{requestNote(r)}</div>
              </div>
              <button type="button" class="fp-btn green" onclick={() => acceptRequest(r.username)}>{t('friends.accept')}</button>
              <button type="button" class="fp-btn red" onclick={() => declineRequest(r.username)}>{t('friends.decline')}</button>
            </div>
          {/each}
        {:else}
          {#each friends.notifications as n (n.id)}
            <div class="fp-row">
              <span class="fp-noteDot" class:unread={n.unread}></span>
              <div class="fp-who">
                <div class="fp-noteText">{noteText(n)}</div>
                <div class="fp-sub">{noteTime(n, now)}</div>
              </div>
              <button type="button" class="fp-dismiss" onclick={() => dismissNote(n.id)}>✕</button>
            </div>
          {/each}
        {/if}
        {#if list.length === 0}
          <div class="fp-empty">{t('friends.empty')}</div>
        {/if}
      </div>

      <div class="fp-foot">
        <span class="fp-toast" class:error={friends.toastError}>{friends.toast}</span>
        <span class="fp-grow"></span>
        {#if friends.tab === 'notes' && friends.notifications.length}
          <button type="button" class="fp-readAll" onclick={readAllNotes}>{t('friends.readAll')}</button>
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .fp-wrap {
    position: absolute; inset: 0; z-index: 43; overflow-y: auto; padding: 20px;
    display: flex; align-items: center; justify-content: center; background: #04040ccc;
  }
  .fp-panel {
    width: 100%; max-width: 560px; max-height: calc(100% - 20px); display: flex; flex-direction: column; position: relative;
    background: linear-gradient(160deg, #3b4252, #1a1f2b 40%, #12161f); border: 1px solid #566078; padding: 18px;
    box-shadow: inset 0 0 0 5px #151a24, inset 0 0 0 6px #3a4254, 0 40px 90px -20px #000, 0 0 60px -30px #ff2a3c;
  }
  .fp-head { display: flex; align-items: center; gap: 12px; }
  .fp-title { font-family: 'Unbounded ExtraBold'; font-size: 16px; letter-spacing: .1em; color: #eef0f5; }
  .fp-online { font-family: 'JetBrains Mono'; font-size: 10px; letter-spacing: .2em; color: #fcee0a; }
  .fp-grow { flex: 1; }
  .fp-close {
    width: 28px; height: 28px; display: grid; place-items: center; border: 1px solid #6b5c05; background: transparent;
    color: #fcee0a; font-size: 12px; cursor: pointer;
  }
  .fp-close:hover { background: #fcee0a14; }
  .fp-hazard { height: 8px; margin: 12px 0; background: repeating-linear-gradient(-45deg, #fcee0a 0 8px, #14161c 8px 16px); opacity: .9; }
  .fp-tabs { display: flex; gap: 6px; }
  .fp-tab {
    display: flex; align-items: center; gap: 7px; padding: 7px 12px; cursor: pointer;
    font-family: 'JetBrains Mono'; font-size: 10.5px; letter-spacing: .14em;
    border: 1px solid #3a4254; color: #8d96a8; background: transparent;
  }
  .fp-tab.active { border-color: #FF55FF; color: #fff; background: #FF55FF22; }
  .fp-tabCount { color: #fcee0a; }
  .fp-addRow { margin-top: 12px; display: flex; gap: 8px; }
  .fp-addBox {
    flex: 1; display: flex; align-items: center; gap: 8px; padding: 0 12px; height: 36px;
    background: #0a0d14; box-shadow: inset 0 0 0 1px #3a4254; cursor: text;
  }
  .fp-plus { color: #7d8699; font-size: 12px; }
  .fp-addBox input {
    flex: 1; min-width: 0; background: transparent; border: none; outline: none; padding: 0;
    color: #eef0f5; font-family: 'JetBrains Mono'; font-size: 12px;
  }
  .fp-addBox input::placeholder { color: #7d585e; }
  .fp-send {
    display: flex; align-items: center; padding: 0 14px; cursor: pointer;
    font-family: 'JetBrains Mono'; font-size: 11px; letter-spacing: .16em;
    border: 1px solid #00ff9f88; color: #00ff9f; background: #00ff9f12;
  }
  .fp-send:hover { background: #00ff9f30; }
  .fp-list { margin-top: 12px; min-height: 240px; overflow-y: auto; display: flex; flex-direction: column; gap: 7px; }
  .fp-row { display: flex; align-items: center; gap: 11px; padding: 9px 11px; background: #0a0d14; box-shadow: inset 0 0 0 1px #3a4254; }
  .fp-avatar { width: 26px; height: 26px; flex: none; display: block; overflow: hidden; }
  .fp-avatar.off { opacity: .6; }
  .fp-who { min-width: 0; flex: 1; }
  .fp-name { font-family: 'JetBrains Mono'; font-size: 12.5px; color: #eef0f5; }
  .fp-sub { font-family: 'JetBrains Mono'; font-size: 9.5px; letter-spacing: .1em; color: #7d8699; margin-top: 3px; }
  .fp-dot { width: 6px; height: 6px; border-radius: 50%; flex: none; background: #5c6578; }
  .fp-dot.on { background: #00ff9f; box-shadow: 0 0 8px #00ff9f; }
  .fp-dot.away { background: #fcee0a; box-shadow: 0 0 8px #fcee0a; }
  .fp-btn {
    padding: 6px 10px; cursor: pointer; white-space: nowrap;
    font-family: 'JetBrains Mono'; font-size: 10px; letter-spacing: .14em; border: 1px solid;
  }
  .fp-btn.yellow { border-color: #fcee0a66; color: #fcee0a; background: #fcee0a10; }
  .fp-btn.yellow:hover { background: #fcee0a30; color: #fff; }
  .fp-btn.green { border-color: #00ff9f66; color: #00ff9f; background: #00ff9f10; }
  .fp-btn.green:hover { background: #00ff9f30; color: #fff; }
  .fp-btn.red { border-color: #ff5a6a66; color: #ff5a6a; background: #ff5a6a10; }
  .fp-btn.red:hover { background: #ff5a6a30; color: #fff; }
  .fp-noteDot { width: 7px; height: 7px; flex: none; background: #3a4254; }
  .fp-noteDot.unread { background: #FF55FF; box-shadow: 0 0 8px #FF55FF; }
  .fp-noteText { font-family: 'Manrope SemiBold'; font-size: 13px; color: #eef0f5; line-height: 1.4; }
  .fp-dismiss {
    width: 24px; height: 24px; display: grid; place-items: center; flex: none; cursor: pointer;
    border: 1px solid #3a4254; background: transparent; color: #7d8699; font-size: 11px;
  }
  .fp-dismiss:hover { border-color: #ff5a6a; color: #ff5a6a; }
  .fp-empty { padding: 60px 0; text-align: center; font-family: 'JetBrains Mono'; font-size: 11px; letter-spacing: .2em; color: #7d8699; }
  .fp-foot {
    display: flex; align-items: center; gap: 10px; margin-top: 12px; min-height: 13px;
    font-family: 'JetBrains Mono'; font-size: 9.5px; letter-spacing: .18em;
  }
  .fp-toast { color: #00ff9f; }
  .fp-toast.error { color: #ff5a6a; }
  .fp-readAll { cursor: pointer; color: #fcee0a; background: none; border: none; padding: 0; font: inherit; letter-spacing: inherit; }
</style>
