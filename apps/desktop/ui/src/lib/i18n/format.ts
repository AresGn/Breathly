import type { Locale } from './catalog.js';

const BYTE_UNITS = ['byte', 'kilobyte', 'megabyte', 'gigabyte', 'terabyte'] as const;
const STEP = 1000;

export function formatNumber(value: number, locale: Locale): string {
	return new Intl.NumberFormat(locale).format(value);
}

function roundTo(value: number, digits: number): number {
	const factor = 10 ** digits;
	return Math.round(value * factor) / factor;
}

// Go et Mo en français, GB et MB en anglais : le nom de l'unité vient d'`Intl`.
// L'unité est choisie sur la valeur arrondie : 999 950 octets font « 1 Mo », pas « 1 000 ko ».
export function formatBytes(bytes: number, locale: Locale): string {
	let index = 0;
	while (
		index < BYTE_UNITS.length - 1 &&
		roundTo(bytes / STEP ** index, digitsFor(index)) >= STEP
	) {
		index += 1;
	}
	return new Intl.NumberFormat(locale, {
		style: 'unit',
		unit: BYTE_UNITS[index],
		unitDisplay: 'short',
		maximumFractionDigits: digitsFor(index)
	}).format(bytes / STEP ** index);
}

function digitsFor(unitIndex: number): number {
	return unitIndex === 0 ? 0 : 1;
}

export function selectPlural(count: number, locale: Locale): 'one' | 'other' {
	return new Intl.PluralRules(locale).select(count) === 'one' ? 'one' : 'other';
}
