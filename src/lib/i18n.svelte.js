// Строки - те же, что у JavaFX-лаунчера (runtime_<язык>.properties -> strings.<язык>.json, tools/props2json.mjs).
// Язык переключается сразу (LanguageSelector -> gui.reload): t() читает реактивную локаль.
import ru from './strings.ru.json';
import en from './strings.en.json';
import be from './strings.be.json';
import uk from './strings.uk.json';
import pl from './strings.pl.json';

const packs = { ru, en, be, uk, pl };

// RuntimeSettings.LAUNCHER_LOCALE: имя в настройках, код пакета, подпись в списке
export const LOCALES = [
  { id: 'RUSSIAN', code: 'ru', name: 'Русский' },
  { id: 'BELARUSIAN', code: 'be', name: 'Беларуская' },
  { id: 'UKRAINIAN', code: 'uk', name: 'Українська' },
  { id: 'POLISH', code: 'pl', name: 'Polska' },
  { id: 'ENGLISH', code: 'en', name: 'English' },
];

export const i18n = $state({ locale: 'RUSSIAN' });

export function t(key, fallback) {
  const code = (LOCALES.find(l => l.id === i18n.locale) || LOCALES[0]).code;
  const v = packs[code][key] ?? ru[key];
  return v !== undefined ? v : fallback !== undefined ? fallback : `'${key}'`;
}

// FxComponent.errorHandle: "runtime.request.<msg>" если есть перевод, иначе сам текст
export function requestError(message) {
  return t('request.' + message, message);
}
