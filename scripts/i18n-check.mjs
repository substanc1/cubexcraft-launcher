// Проверка i18n лаунчера. Запуск: node scripts/i18n-check.mjs (из корня проекта или откуда угодно).
// Без зависимостей. Код выхода: 0 - всё в порядке, 1 - есть ошибки.
//
// Что проверяем:
//  1. словари en/uk/be/pl содержат все ключи ru и без лишних;
//  2. нет пустых значений (кроме EMPTY_OK - см. комментарий там);
//  3. плейсхолдеры {var} в переводах совпадают с ru по набору;
//  4. в uk нет букв ы э ъ ё; в en нет кириллицы;
//  5. в src/**/*.{svelte,js} (кроме src/lib/i18n.svelte.js) после удаления комментариев
//     нет строк с кириллицей, кроме явных исключений ALLOW.

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const CODES = ['ru', 'en', 'uk', 'be', 'pl'];
const CYR = /\p{Script=Cyrillic}/u;

// Исключения из проверки пустых значений. Сейчас пусто: пустые ключи, которых нет в коде, удалены (SCRUM-734).
const EMPTY_OK = new Set([]);

// Русский текст, который осознанно остаётся в en/uk: имя кнопки в боте «✅ Это я» (на русском у бота).
// Для проверок буквы ы/э/ъ/ё (uk) и кириллицы (en) такой ключ пропускается.
const RU_LABEL_OK = new Set(['overlay.processing.text.twofa']);

// Разрешённая кириллица в коде. Причина у каждой записи.
// file - путь от корня, match - подстрока строки (после trim), reason - почему это не строка интерфейса.
const ALLOW = [
  { file: 'src/lib/backend.js', match: "info: 'CubeXTech — приватный сервер", reason: 'моковое описание сервера (режим без ядра)' },
  { file: 'src/lib/backend.js', match: "title: 'CubeXCraft теперь на телефоне'", reason: 'моковая новость' },
  { file: 'src/lib/backend.js', match: "title: 'Скидки в честь открытия", reason: 'моковая новость' },
  { file: 'src/lib/backend.js', match: "title: 'Голосуй за проект", reason: 'моковая новость' },
  { file: 'src/scenes/Login.svelte', match: "call('timing_mark', { what: 'меню серверов показано' })", reason: 'внутренняя метка замера, в интерфейс не выводится' },
];

const errors = [];
const info = [];
const dicts = {};
for (const c of CODES) {
  dicts[c] = JSON.parse(fs.readFileSync(path.join(ROOT, 'src/lib', `strings.${c}.json`), 'utf8'));
}
const ru = dicts.ru;
const ruKeys = Object.keys(ru);
const phs = s => [...new Set(String(s).match(/\{[^{}]+\}/g) || [])].sort().join(' ');

// 1-2: ключи и пустые значения
for (const c of CODES) {
  const d = dicts[c];
  if (c !== 'ru') {
    const missing = ruKeys.filter(k => !(k in d));
    const extra = Object.keys(d).filter(k => !(k in ru));
    for (const k of missing) errors.push(`[${c}] нет ключа из ru: ${k}`);
    for (const k of extra) errors.push(`[${c}] лишний ключ (нет в ru): ${k}`);
  }
  for (const [k, v] of Object.entries(d)) {
    if (typeof v !== 'string') errors.push(`[${c}] значение не строка: ${k}`);
    else if (v.trim() === '' && !EMPTY_OK.has(k)) errors.push(`[${c}] пустое значение: ${k}`);
  }
}
info.push(
  `ключей: ru=${ruKeys.length}, ` +
    CODES.filter(c => c !== 'ru')
      .map(c => `${c}=${Object.keys(dicts[c]).length}`)
      .join(', '),
);
info.push(`пустых значений-исключений: ${EMPTY_OK.size} (см. EMPTY_OK)`);

// 3: плейсхолдеры
for (const c of CODES) {
  if (c === 'ru') continue;
  for (const k of ruKeys) {
    if (!(k in dicts[c])) continue;
    if (phs(dicts[c][k]) !== phs(ru[k])) errors.push(`[${c}] плейсхолдеры не совпадают с ru: ${k} (ru: "${phs(ru[k])}", ${c}: "${phs(dicts[c][k])}")`);
  }
}

// 4: буквы по языкам
for (const [k, v] of Object.entries(dicts.uk)) {
  if (!RU_LABEL_OK.has(k) && /[ыэъёЫЭЪЁ]/.test(v)) errors.push(`[uk] русская буква (ыэъё) в значении: ${k}`);
}
for (const [k, v] of Object.entries(dicts.en)) {
  if (!RU_LABEL_OK.has(k) && CYR.test(v)) errors.push(`[en] кириллица в значении: ${k}`);
}

// 5: кириллица в коде
function walk(dir, out) {
  for (const f of fs.readdirSync(dir)) {
    const p = path.join(dir, f);
    if (fs.statSync(p).isDirectory()) walk(p, out);
    else if (/\.(svelte|js)$/.test(f)) out.push(p);
  }
}
// комментарии заменяются пустотой с сохранением переводов строк (номера строк остаются верными)
function stripComments(text) {
  const keepNl = m => m.replace(/[^\n]/g, '');
  let s = text.replace(/\/\*[\s\S]*?\*\//g, keepNl).replace(/<!--[\s\S]*?-->/g, keepNl);
  return s
    .split('\n')
    .map(line => line.replace(/(^|[^:\\])\/\/.*$/, '$1'))
    .join('\n');
}
const files = [];
walk(path.join(ROOT, 'src'), files);
let codeLines = 0;
let allowed = 0;
const used = new Set();
for (const abs of files) {
  const rel = path.relative(ROOT, abs).split(path.sep).join('/');
  if (rel === 'src/lib/i18n.svelte.js') continue;
  const orig = fs.readFileSync(abs, 'utf8').split(/\r?\n/);
  const stripped = stripComments(orig.join('\n')).split('\n');
  stripped.forEach((line, i) => {
    if (!CYR.test(line)) return;
    codeLines++;
    const src = orig[i].trim();
    const rule = ALLOW.find(a => a.file === rel && src.includes(a.match));
    if (rule) {
      allowed++;
      used.add(rule);
      return;
    }
    errors.push(`${rel}:${i + 1}: кириллица вне словаря: ${src.slice(0, 160)}`);
  });
}
info.push(`кириллических строк в коде: ${codeLines}, из них по исключениям: ${allowed}`);
for (const a of ALLOW) if (!used.has(a)) errors.push(`исключение не сработало (строка ушла или изменилась): ${a.file}: ${a.match}`);

// итог
for (const line of info) console.log('  ' + line);
if (errors.length) {
  console.log('');
  for (const e of errors) console.log('ОШИБКА: ' + e);
  console.log(`\ni18n-check: FAIL (ошибок: ${errors.length})`);
  process.exit(1);
}
console.log('\ni18n-check: OK');
process.exit(0);
