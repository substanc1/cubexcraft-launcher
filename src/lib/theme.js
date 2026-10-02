// Пользовательские цвета интерфейса - формулы CustomThemeHelper.buildCss/fit* старого лаунчера 1:1.
// accent/background/text/lines (hex) -> производные токены палитры; пусто = стандартная палитра.

export function hexToRgb(hex) {
  const m = /^#?([0-9a-f]{6})$/i.exec(String(hex || '').trim());
  if (!m) return null;
  const n = parseInt(m[1], 16);
  return [(n >> 16) & 255, (n >> 8) & 255, n & 255];
}
const toHex = ([r, g, b]) => '#' + [r, g, b].map(v => Math.round(Math.max(0, Math.min(255, v))).toString(16).padStart(2, '0')).join('').toUpperCase();
const toRgba = ([r, g, b], a) => `rgba(${Math.round(r)}, ${Math.round(g)}, ${Math.round(b)}, ${a.toFixed(2)})`;

// JavaFX Color.getHue / Color.hsb
export function hue([r, g, b]) {
  r /= 255; g /= 255; b /= 255;
  const max = Math.max(r, g, b), min = Math.min(r, g, b), d = max - min;
  if (d === 0) return 0;
  let h = max === r ? (g - b) / d : max === g ? 2 + (b - r) / d : 4 + (r - g) / d;
  h *= 60;
  return h < 0 ? h + 360 : h;
}
export function hsb(h, s, v) {
  const c = v * s, x = c * (1 - Math.abs(((h / 60) % 2) - 1)), m = v - c;
  const [r, g, b] = h < 60 ? [c, x, 0] : h < 120 ? [x, c, 0] : h < 180 ? [0, c, x] : h < 240 ? [0, x, c] : h < 300 ? [x, 0, c] : [c, 0, x];
  return [(r + m) * 255, (g + m) * 255, (b + m) * 255];
}
const fit = (rgb, s, v) => hsb(hue(rgb), s, v);

export const fitAccent = rgb => toHex(fit(rgb, 0.84, 1.0));
export const fitBackground = rgb => toHex(fit(rgb, 0.6, 0.16));
export const fitText = rgb => toHex(fit(rgb, 0.3, 0.95));
export const fitLines = rgb => toHex(fit(rgb, 0.78, 0.34));
export const DEFAULTS = { accent: '#FF2A3C', background: '#04040C', text: '#FAEEEE', lines: '#4A1018' };

export function themeVars(c) {
  const v = {};
  let x;
  if ((x = hexToRgb(c.accent))) {
    v['--red'] = toHex(x);
    v['--red-hover'] = toHex(x.map(k => k + (255 - k) * 0.2));
    v['--redTr'] = toRgba(x, 0.12);
    v['--redTr-hover'] = toRgba(x.map(k => k + (255 - k) * 0.4), 0.12);
    v['--red-ghost'] = toRgba(x, 0.53);
    v['--glow-color'] = toRgba(x, 0.53);
  }
  if ((x = hexToRgb(c.background))) {
    const c2 = toHex(fit(x, 0.6, 0.416)), c3 = toHex(fit(x, 0.6, 0.08));
    v['--base'] = toHex(x);
    v['--baseG'] = c2;
    v['--panel-flat'] = toHex(x);
    v['--gray'] = toHex(x);
    v['--panel-focus'] = c2;
    v['--panel'] = `linear-gradient(to bottom right, ${c2} 0%, ${c3} 100%)`;
  }
  if ((x = hexToRgb(c.text))) {
    v['--text'] = toHex(x);
    v['--text-hover'] = toHex(fit(x, 0.3, 1.0));
    v['--header'] = toHex(fit(x, 0.38, 0.85));
    v['--header-hover'] = toHex(fit(x, 0.3, 1.0));
    v['--text-secondary'] = toHex(fit(x, 0.35, 0.65));
  }
  if ((x = hexToRgb(c.lines))) {
    v['--border-dark'] = toHex(x);
    v['--border-light'] = toHex(fit(x, 0.78, 0.561));
    v['--glass'] = toRgba(x, 0.28);
  }
  return v;
}

const ALL = ['--red', '--red-hover', '--redTr', '--redTr-hover', '--red-ghost', '--glow-color', '--base', '--baseG', '--panel-flat', '--gray',
  '--panel-focus', '--panel', '--text', '--text-hover', '--header', '--header-hover', '--text-secondary', '--border-dark',
  '--border-light', '--glass'];

export function applyTheme(settings) {
  const root = document.documentElement;
  for (const k of ALL) root.style.removeProperty(k);
  const vars = themeVars((settings && settings.customColors) || {});
  for (const [k, val] of Object.entries(vars)) root.style.setProperty(k, val);
}
