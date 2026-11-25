// Разметка акцентов в строках: **жирный** -> kind 'b', ==неон== -> kind 'neon'.
// Без вложенности. Незакрытый маркер остаётся обычным текстом (весь хвост от него).
// Пустые токены не выдаются, соседние text склеиваются.

export function parseRich(str) {
  if (typeof str !== 'string' || str === '') return [];
  const out = [];
  let buf = '';
  const flush = () => {
    if (buf !== '') out.push({ kind: 'text', text: buf });
    buf = '';
  };
  const n = str.length;
  let i = 0;
  while (i < n) {
    let marker = null;
    let kind = null;
    if (str.startsWith('**', i)) { marker = '**'; kind = 'b'; }
    else if (str.startsWith('==', i)) { marker = '=='; kind = 'neon'; }
    if (marker) {
      const j = str.indexOf(marker, i + 2);
      if (j === -1) {
        buf += str.slice(i);
        break;
      }
      const inner = str.slice(i + 2, j);
      if (inner !== '') {
        flush();
        out.push({ kind, text: inner });
      }
      i = j + 2;
      continue;
    }
    buf += str[i];
    i++;
  }
  flush();
  return out;
}

export function plainRich(str) {
  if (typeof str !== 'string') return '';
  return parseRich(str).map((tok) => tok.text).join('');
}
