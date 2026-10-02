// Плавные переходы между экранами и появление диалогов. Только opacity/transform - считается на GPU, не грузит слабые ПК.
// «Уменьшить эффекты» - короткое затухание без движения.
import { cubicOut } from 'svelte/easing';
import { app } from './app.svelte.js';

const reduced = () => !!(app.settings && app.settings.reduceEffects);

// новый экран проявляется поверх старого: вперёд - «наплывает» из чуть меньшего масштаба, назад - из чуть большего
export function sceneIn(node) {
  if (reduced()) return { duration: 140, css: t => `opacity:${t}` };
  const from = app.navDir < 0 ? 1.025 : 0.975;
  return {
    duration: 320,
    easing: cubicOut,
    css: t => `opacity:${t}; transform: scale(${from + (1 - from) * t})`,
  };
}

// старый экран остаётся под новым, слегка темнея и уходя «вглубь»; кликов уже не принимает
export function sceneOut(node) {
  node.style.zIndex = '0';
  node.style.pointerEvents = 'none';
  if (reduced()) return { duration: 140, css: () => 'opacity:1' };
  const to = app.navDir < 0 ? 0.985 : 1.015;
  // для ухода Svelte считает t = 1 - easing(время): cubicOut - старый экран гаснет сразу, без «двоения» с новым
  return {
    duration: 320,
    easing: cubicOut,
    css: (t, u) => `opacity:${0.2 + 0.8 * t}; transform: scale(${1 + (to - 1) * u})`,
  };
}

// диалоги/оверлеи: фон затемняется, карточка всплывает
export function popIn(node, { delay = 0 } = {}) {
  if (reduced()) return { duration: 120, css: t => `opacity:${t}` };
  return { delay, duration: 220, easing: cubicOut, css: t => `opacity:${t}; transform: translateY(${(1 - t) * 10}px) scale(${0.97 + 0.03 * t})` };
}
export function popOut() {
  if (reduced()) return { duration: 100, css: t => `opacity:${t}` };
  return { duration: 160, easing: cubicOut, css: t => `opacity:${t}; transform: scale(${0.98 + 0.02 * t})` };
}
export function fadeQuick(node, { duration = 200 } = {}) {
  return { duration: reduced() ? 100 : duration, easing: cubicOut, css: t => `opacity:${t}` };
}
