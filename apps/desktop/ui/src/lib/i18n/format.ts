import type { Locale } from './catalog.js';

const BYTE_UNITS = ['byte', 'kilobyte', 'megabyte', 'gigabyte', 'terabyte'] as const;
const STEP = 1000;

export function formatNumber(value: number, locale: Locale): string {
	return new Intl.NumberFormat(locale).format(value);
}

// Go et Mo en français, GB et MB en anglais : le nom de l'unité vient d'`Intl`.
export function formatBytes(bytes: number, locale: Locale): string {
	let value = bytes;
	let index = 0;
	while (value >= STEP && index < BYTE_UNITS.length - 1) {
		value /= STEP;
		index += 1;
	}
	return new Intl.NumberFormat(locale, {
		style: 'unit',
		unit: BYTE_UNITS[index],
		unitDisplay: 'short',
		maximumFractionDigits: index === 0 ? 0 : 1
	}).format(value);
}

export function selectPlural(count: number, locale: Locale): 'one' | 'other' {
	return new Intl.PluralRules(locale).select(count) === 'one' ? 'one' : 'other';
}
