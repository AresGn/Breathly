import type { Locale, MessageKey } from './catalog.js';
import { pickLocale } from './system-locale.js';
import { translate, type Values } from './translate.js';

function systemLanguages(): readonly string[] {
	return typeof navigator === 'undefined' ? [] : navigator.languages;
}

let current = $state<Locale>(pickLocale(systemLanguages()));

export function getLocale(): Locale {
	return current;
}

// Lire `t` dans un composant le rend réactif : changer la langue met à jour les textes sans redémarrer.
export function t(key: MessageKey, values?: Values): string {
	return translate(current, key, values);
}

export function setLocale(locale: Locale): void {
	current = locale;
}

export function resetLocale(): void {
	current = pickLocale(systemLanguages());
}
