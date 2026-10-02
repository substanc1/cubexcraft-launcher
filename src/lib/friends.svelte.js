// Друзья (макет handoff-10): список, заявки, уведомления, статус. Данные - снимок с сайта (launcher-friends.js):
// приходит событием friends-update от ядра раз в ~30 с (вместе с отправкой своего статуса) и в ответ на действия.
// Свой статус: online - лаунчер открыт, away - 5 мин без действий в окне лаунчера или окно скрыто/свёрнуто;
// «играет на сборке» ставит ядро само при запуске игры.
import { app, t, notify } from './app.svelte.js';
import { call, on } from './backend.js';

const AWAY_MS = 5 * 60 * 1000;

export const friends = $state({
  open: false,
  tab: 'friends',
  friends: [],
  requests: [],
  notifications: [],
  receivedAt: 0, // когда пришёл снимок (длительность игры друга досчитывается от него)
  toast: '',
  toastError: false,
});

const fmt = (s, vars) => Object.entries(vars).reduce((acc, [k, v]) => acc.split('{' + k + '}').join(v), s);

let seenNotes = null; // id уведомлений и ники заявок, уже показанные всплывашкой
function apply(v) {
  if (!v || !Array.isArray(v.friends)) return;
  // новое приглашение или заявка, пока окно друзей закрыто - всплывающее уведомление лаунчера
  const keys = [...v.notifications.map(n => 'n' + n.id), ...v.requests.map(r => 'r' + r.username)];
  if (seenNotes && !friends.open) {
    for (const n of v.notifications) {
      if (n.unread && n.kind === 'invite' && !seenNotes.has('n' + n.id)) notify(t('friends.inviteHead'), noteText(n));
    }
    for (const r of v.requests) {
      if (!seenNotes.has('r' + r.username)) notify(t('friends.requestHead'), fmt(t('friends.requestText'), { a: r.username }));
    }
  }
  seenNotes = new Set(keys);
  friends.friends = v.friends;
  friends.requests = v.requests;
  friends.notifications = v.notifications;
  friends.receivedAt = Date.now();
}

function toast(text, error = false) {
  friends.toast = text;
  friends.toastError = error;
}

async function act(promise, okText) {
  try {
    const v = await promise;
    apply(v);
    if (okText) toast(okText(v));
    return v;
  } catch (e) {
    toast(String(e && (e.message || e)), true);
    return null;
  }
}

export function openFriends(tab) {
  friends.open = true;
  if (tab) friends.tab = tab;
  friends.toast = '';
  refreshFriends();
}
export function closeFriends() {
  friends.open = false;
  friends.toast = '';
}

export function refreshFriends() {
  return call('friends_get').then(apply).catch(() => {});
}

const actFriend = (action, username, extra = {}) => call('friends_action', { action, username, server: null, ...extra });

export const sendRequest = name =>
  act(actFriend('request', name), v => fmt(t(v.result === 'friends' ? 'friends.tFriends' : 'friends.tSent'), { n: v.username }));
export const acceptRequest = name => act(actFriend('accept', name), v => fmt(t('friends.tFriends'), { n: v.username }));
export const declineRequest = name => act(actFriend('decline', name), v => fmt(t('friends.tDeclined'), { n: v.username }));
export const removeFriend = name => act(actFriend('remove', name), v => fmt(t('friends.tRemoved'), { n: v.username }));
export function inviteFriend(f, server) {
  if (f.state === 'offline') return toast(fmt(t('friends.tOffline'), { n: f.username }));
  return act(actFriend('invite', f.username, { server }), v => fmt(t('friends.tInvited'), { n: v.username }));
}
export const readAllNotes = () => act(call('notes_action', { action: 'read', id: null }), () => t('friends.tRead'));
export const dismissNote = id => act(call('notes_action', { action: 'dismiss', id }));

// ---- тексты

function duration(sec) {
  const m = Math.floor(sec / 60);
  const h = Math.floor(m / 60);
  return h ? `${h}${t('friends.h')} ${m % 60}${t('friends.m')}` : `${Math.max(1, m)}${t('friends.m')}`;
}

/** «CubeXCraft · 1ч 20м» / «в сети» / «отошёл» / «не в сети»; now - для тиканья длительности */
export function statusText(f, now = Date.now()) {
  if (f.state === 'playing') return `${f.server} · ${duration(f.playingSec + Math.max(0, (now - friends.receivedAt) / 1000))}`;
  return t(f.state === 'online' ? 'friends.stOnline' : f.state === 'away' ? 'friends.stAway' : 'friends.stOffline');
}

export function noteText(n) {
  const key = n.kind === 'invite' ? 'friends.nInvite' : n.kind === 'accepted' ? 'friends.nAccepted' : 'friends.nPlaying';
  return fmt(t(key), { a: n.actor, s: n.server });
}

export function noteTime(n, now = Date.now()) {
  const sec = n.agoSec + Math.max(0, (now - friends.receivedAt) / 1000);
  if (sec < 60) return t('friends.now');
  if (sec < 3600) return fmt(t('friends.min'), { n: Math.floor(sec / 60) });
  const at = new Date(Date.now() - sec * 1000);
  const hm = at.toLocaleTimeString('ru-RU', { hour: '2-digit', minute: '2-digit' });
  const today = new Date();
  const yesterday = new Date(today.getFullYear(), today.getMonth(), today.getDate() - 1);
  if (at.toDateString() === today.toDateString()) return fmt(t('friends.hours'), { n: Math.floor(sec / 3600) });
  if (at.toDateString() === yesterday.toDateString()) return fmt(t('friends.yesterday'), { t: hm });
  return at.toLocaleDateString('ru-RU', { day: '2-digit', month: '2-digit' }) + ' · ' + hm;
}

export const requestNote = r => (r.mutual > 0 ? fmt(t('friends.mutual'), { n: r.mutual }) : t('friends.wants'));

// ---- свой статус: online / away

let lastInput = Date.now();
let sentUi = '';
function pushUi() {
  const ui = document.hidden || Date.now() - lastInput > AWAY_MS ? 'away' : 'online';
  if (ui !== sentUi && app.user) {
    sentUi = ui;
    call('presence_set', { ui }).catch(() => {});
  }
}

export async function initFriends() {
  await on('friends-update', apply);
  for (const ev of ['mousemove', 'mousedown', 'keydown', 'wheel']) {
    window.addEventListener(ev, () => {
      lastInput = Date.now();
      if (sentUi === 'away') pushUi();
    }, { passive: true });
  }
  document.addEventListener('visibilitychange', pushUi);
  setInterval(pushUi, 15000);
}

/** вошли в аккаунт (меню серверов): сразу статус «в сети» и свежий снимок */
export function friendsOnLogin() {
  seenNotes = null; // другой аккаунт - без всплывашек о «старых» приглашениях
  sentUi = '';
  lastInput = Date.now();
  pushUi();
  refreshFriends();
}
