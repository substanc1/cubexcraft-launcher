// Полоса прокрутки как у JavaFX-лаунчера (global.css ScrollBar): своя дорожка (стекло, тень, радиус 10) и бегунок
// (-fx-border-dark, отступ 1, красный при наведении). Нативная полоса скрыта. Позиция задаётся относительно
// прокручиваемого блока: right (отрицательный - вынесена за рамку, как translate-x: 17px), top/bottom - отступы
// дорожки (padding ScrollBar + кнопки-стрелки). Видна только когда содержимое не помещается (AS_NEEDED).
// use:fxscroll={{ right: -15, top: 110, bottom: 111, width: 12 }}
export function fxscroll(node, opts) {
  let o = { right: 0, top: 4, bottom: 4, width: 12, ...opts };
  node.classList.add('fx-scroll');
  const track = document.createElement('div');
  track.className = 'fx-track';
  const thumb = document.createElement('div');
  thumb.className = 'fx-thumb';
  track.appendChild(thumb);
  const host = node.parentElement;
  host.appendChild(track);

  function layout() {
    const visible = node.clientHeight, total = node.scrollHeight;
    if (total <= visible + 1) {
      track.style.display = 'none';
      return;
    }
    track.style.display = 'block';
    track.style.left = node.offsetLeft + node.offsetWidth - o.width - o.right + 'px';
    track.style.top = node.offsetTop + o.top + 'px';
    track.style.width = o.width + 'px';
    const len = node.offsetHeight - o.top - o.bottom;
    track.style.height = len + 'px';
    const th = Math.max(16, (len * visible) / total);
    const pos = ((len - th) * node.scrollTop) / (total - visible);
    thumb.style.height = th + 'px';
    thumb.style.top = pos + 'px';
  }

  let drag = null;
  thumb.addEventListener('mousedown', e => {
    e.preventDefault();
    e.stopPropagation();
    drag = { y: e.clientY, top: node.scrollTop };
    thumb.classList.add('pressed');
  });
  const move = e => {
    if (!drag) return;
    const len = track.clientHeight, th = thumb.offsetHeight;
    const k = (node.scrollHeight - node.clientHeight) / Math.max(1, len - th);
    node.scrollTop = drag.top + (e.clientY - drag.y) * k;
  };
  const up = () => {
    drag = null;
    thumb.classList.remove('pressed');
  };
  track.addEventListener('mousedown', e => {
    if (e.target !== track) return;
    e.preventDefault();
    e.stopPropagation();
    const r = thumb.getBoundingClientRect();
    node.scrollTop += (e.clientY < r.top ? -1 : 1) * node.clientHeight;
  });
  window.addEventListener('mousemove', move);
  window.addEventListener('mouseup', up);
  node.addEventListener('scroll', layout);
  const ro = new ResizeObserver(layout);
  ro.observe(node);
  if (node.firstElementChild) ro.observe(node.firstElementChild);
  requestAnimationFrame(layout);

  return {
    update(n) {
      o = { ...o, ...n };
      layout();
    },
    destroy() {
      ro.disconnect();
      window.removeEventListener('mousemove', move);
      window.removeEventListener('mouseup', up);
      track.remove();
    },
  };
}
