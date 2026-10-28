import { FALLBACK_LOCALE, isLocale, type Locale } from './catalog.js';

// Première langue préférée prise en charge ("fr-FR" vaut "fr"), sinon l'anglais.
export function pickLocale(preferred: readonly string[]): Locale {
	for (const tag of preferred) {
		const language = tag.toLowerCase().split('-')[0] ?? '';
		if (isLocale(language)) return language;
	}
	return FALLBACK_LOCALE;
}
