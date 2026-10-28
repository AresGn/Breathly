import enJson from '../../../../../../locales/en.json';
import frJson from '../../../../../../locales/fr.json';

export type Message = string | { readonly one: string; readonly other: string };

export const LOCALES = ['fr', 'en'] as const;
export type Locale = (typeof LOCALES)[number];
export const FALLBACK_LOCALE: Locale = 'en';

// `fr.json` fixe l'ensemble des clés : `en.json` doit les avoir toutes, sinon le typage échoue.
export type MessageKey = keyof typeof frJson;
type Catalog = Readonly<Record<MessageKey, Message>>;

const fr: Catalog = frJson;
const en: Catalog = enJson;

export const CATALOGS: Readonly<Record<Locale, Catalog>> = { fr, en };

export function isLocale(value: string): value is Locale {
	return LOCALES.some((locale) => locale === value);
}
