// Журнал игры для экрана debug. Строка NeoForge «[15:39:10] [Render thread/WARN] [mixin/]: текст» разбирается на
// время, поток, уровень, источник и текст; строки без заголовка (стек ошибки, продолжение многострочного сообщения)
// прикрепляются к предыдущей записи. Подряд идущие одинаковые сообщения схлопываются в одну запись со счётчиком.
// Записи - обычный массив (не $state: тысячи строк в секунду при старте), интерфейс обновляется по log.rev,
// не чаще раза за кадр. Буфер ~256К символов, старые записи отбрасываются целиком.
export const LEVELS = ['error', 'warn', 'info', 'debug', 'other'];

export const log = $state({
  rev: 0,
  counts: { error: 0, warn: 0, info: 0, debug: 0, other: 0 },
  total: 0,
  exit: null, // { code, killed, saved } - игра завершилась
});

/** @type {{ id: number, time: string, thread: string, level: string, tag: string, src: string, msg: string, more: string[], n: number, raw: string, size: number }[]} */
export const entries = [];

const MAX = 262144;
let chars = 0;
let seq = 0;
let frame = 0;

const HEAD = /^\[(\d\d:\d\d:\d\d)(?:[.,]\d+)?\] \[([^\]]*)\/(TRACE|DEBUG|INFO|WARN|ERROR|FATAL)\] \[([^\]]*)\]:? ?(.*)$/;
const TRACE_LINE = /^(\s|at |Caused by|Suppressed:|\.\.\. \d+ more)/;
const EXCEPTION = /^(Exception in thread|[\w$]+(\.[\w$]+)+(Exception|Error|Throwable)\b)/;

// строки без заголовка (System.out модов, java.util.logging, log4j status): уровень по слову в строке
const RAW_LEVEL = [[/\b(SEVERE|ERROR|FATAL)\b/, 'error', 'ERROR'], [/\bWARN(ING)?\b/, 'warn', 'WARN'], [/\bINFO\b/, 'info', 'INFO']];

const BUCKET = { TRACE: 'debug', DEBUG: 'debug', INFO: 'info', WARN: 'warn', ERROR: 'error', FATAL: 'error' };

function bump() {
  if (frame) return;
  frame = requestAnimationFrame(() => {
    frame = 0;
    log.rev++;
  });
}

// «co.ze.ne.st.StorageManager/» -> «StorageManager»; «mixin/» -> «mixin»; «STDOUT/» -> «STDOUT»
function shortSrc(s) {
  const [logger, marker] = s.split('/');
  const last = logger.split('.').pop() || logger;
  return marker ? `${last}/${marker}` : last;
}

function add(e) {
  entries.push(e);
  log.counts[e.level]++;
  log.total++;
  chars += e.size;
}

function trim() {
  while (chars > MAX && entries.length > 1) {
    const e = entries.shift();
    chars -= e.size;
    log.counts[e.level]--;
    log.total--;
  }
}

function push(line) {
  const last = entries[entries.length - 1];
  const m = HEAD.exec(line);
  if (m) {
    const [, time, thread, lvl, src, msg] = m;
    const level = BUCKET[lvl];
    // то же сообщение подряд - счётчик у предыдущей записи
    if (last && last.level === level && last.src === src && last.msg === msg && !last.more.length) {
      last.n++;
      last.time = time;
      return;
    }
    add({ id: ++seq, time, thread, level, tag: lvl, src, short: shortSrc(src), msg, more: [], n: 1, raw: line, size: line.length + 1 });
    return;
  }
  if (last && (TRACE_LINE.test(line) || (EXCEPTION.test(line) && (last.level === 'error' || last.level === 'warn' || last.more.length)))) {
    last.more.push(line);
    last.size += line.length + 1;
    chars += line.length + 1;
    return;
  }
  if (!line.trim()) return;
  let level = 'other', tag = '';
  if (EXCEPTION.test(line)) [level, tag] = ['error', 'ERROR'];
  else for (const [re, lv, tg] of RAW_LEVEL) if (re.test(line)) { [level, tag] = [lv, tg]; break; }
  add({ id: ++seq, time: '', thread: '', level, tag, src: '', short: '', msg: line, more: [], n: 1, raw: line, size: line.length + 1 });
}

export function appendLog(text) {
  for (const line of String(text).split('\n')) push(line.replace(/\r$/, ''));
  trim();
  bump();
}

export function clearLog() {
  entries.length = 0;
  chars = 0;
  seq = 0;
  for (const k of LEVELS) log.counts[k] = 0;
  log.total = 0;
  log.exit = null;
  log.rev++;
}

/** текст записи для копирования: исходные строки, повторы - пометкой */
export function entryText(e) {
  const head = e.n > 1 ? `${e.raw}  (x${e.n})` : e.raw;
  return e.more.length ? head + '\n' + e.more.join('\n') : head;
}
