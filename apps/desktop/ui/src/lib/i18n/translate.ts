import { CATALOGS, type Locale, type Message, type MessageKey } from './catalog.js';
import { formatBytes, formatNumber, selectPlural } from './format.js';

export type Values = Readonly<Record<string, number | string>>;

// Un paramètre `size` (octets) est écrit avec son unité ; les autres nombres par `Intl`.
function renderValue(name: string, value: number | string, locale: Locale): string {
	if (typeof value === 'string') return value;
	return name === 'size' ? formatBytes(value, locale) : formatNumber(value, locale);
}

function pickForm(message: Message, values: Values, locale: Locale): string {
	if (typeof message === 'string') return message;
	const count = values['count'];
	return message[typeof count === 'number' ? selectPlural(count, locale) : 'other'];
}

export function translate(locale: Locale, key: MessageKey, values: Values = {}): string {
	const template = pickForm(CATALOGS[locale][key], values, locale);
	return template.replace(/\{(\w+)\}/g, (placeholder, name: string) => {
		const value = values[name];
		return value === undefined ? placeholder : renderValue(name, value, locale);
	});
}
