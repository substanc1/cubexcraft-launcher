// Tooltip (.tooltip: -fx-show-delay 100ms, фон -fx-colors-base, 12px). Svelte action: use:tooltip={'текст'}
let el = null;
let timer = null;

function hide() {
  clearTimeout(timer);
  if (el) el.remove();
  el = null;
}

export function tooltip(node, text) {
  let current = text;
  function enter(e) {
    if (!current) return;
    clearTimeout(timer);
    timer = setTimeout(() => {
      hide();
      el = document.createElement('div');
      el.className = 'tip';
      el.textContent = current;
      document.body.appendChild(el);
      const r = node.getBoundingClientRect();
      const w = el.offsetWidth;
      // anchorLocation=WINDOW_BOTTOM_LEFT: под элементом, левым краем к нему, не вылезая за окно
      el.style.left = Math.max(4, Math.min(r.left, window.innerWidth - w - 4)) + 'px';
      el.style.top = Math.min(r.bottom + 4, window.innerHeight - el.offsetHeight - 4) + 'px';
    }, 100);
  }
  node.addEventListener('mouseenter', enter);
  node.addEventListener('mouseleave', hide);
  node.addEventListener('mousedown', hide);
  return {
    update(t) { current = t; },
    destroy() {
      node.removeEventListener('mouseenter', enter);
      node.removeEventListener('mouseleave', hide);
      node.removeEventListener('mousedown', hide);
      hide();
    },
  };
}
