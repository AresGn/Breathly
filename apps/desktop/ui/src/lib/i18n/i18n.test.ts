import { afterEach, describe, expect, it } from 'vitest';
import { formatBytes } from './format.js';
import { getLocale, resetLocale, setLocale, t } from './i18n.svelte.js';
import { pickLocale } from './system-locale.js';
import { renderMessage } from './translate.js';

const NBSP = new RegExp('[\u00A0\u202F]', 'g');
const plain = (text: string): string => text.replace(NBSP, ' ');

afterEach(resetLocale);

describe('langue au démarrage', () => {
	it('prend la langue du système si elle est prise en charge', () => {
		expect(pickLocale(['fr-FR', 'en-US'])).toBe('fr');
		expect(pickLocale(['EN-gb'])).toBe('en');
	});

	it('saute les langues non prises en charge', () => {
		expect(pickLocale(['de-DE', 'fr-CA'])).toBe('fr');
	});

	it('retombe sur l’anglais sinon', () => {
		expect(pickLocale(['de-DE', 'ja'])).toBe('en');
		expect(pickLocale([])).toBe('en');
	});
});

describe('unités', () => {
	it('écrit Go et Mo en français', () => {
		expect(plain(formatBytes(2_500_000_000, 'fr'))).toBe('2,5 Go');
		expect(plain(formatBytes(3_000_000, 'fr'))).toBe('3 Mo');
	});

	it('écrit GB et MB en anglais', () => {
		expect(plain(formatBytes(2_500_000_000, 'en'))).toBe('2.5 GB');
		expect(plain(formatBytes(3_000_000, 'en'))).toBe('3 MB');
	});
});

describe('arrondi à la limite d’unité', () => {
	it('passe à l’unité supérieure quand l’arrondi atteint 1 000', () => {
		expect(plain(formatBytes(999_950, 'fr'))).toBe('1 Mo');
		expect(plain(formatBytes(999_960_000, 'en'))).toBe('1 GB');
		expect(plain(formatBytes(999_940, 'en'))).toBe('999.9 kB');
	});
});

describe('pluriels et valeurs', () => {
	const files = { one: '{count} fichier trouvé', other: '{count} fichiers trouvés' };
	const filesEn = { one: '{count} file found', other: '{count} files found' };

	it('accorde selon la langue (0 est singulier en français)', () => {
		expect(renderMessage(files, 'fr', { count: 0 })).toBe('0 fichier trouvé');
		expect(renderMessage(files, 'fr', { count: 1 })).toBe('1 fichier trouvé');
		expect(renderMessage(files, 'fr', { count: 2 })).toBe('2 fichiers trouvés');
		expect(renderMessage(filesEn, 'en', { count: 0 })).toBe('0 files found');
		expect(renderMessage(filesEn, 'en', { count: 1 })).toBe('1 file found');
	});

	it('formate les grands nombres selon la langue', () => {
		expect(renderMessage(filesEn, 'en', { count: 1234 })).toBe('1,234 files found');
	});

	it('insère une taille avec son unité', () => {
		expect(plain(renderMessage('{size} libérés', 'fr', { size: 2e9 }))).toBe('2 Go libérés');
		expect(plain(renderMessage('{size} freed', 'en', { size: 2e9 }))).toBe('2 GB freed');
	});
});

describe('changement de langue à chaud', () => {
	it('change les textes sans redémarrer', () => {
		setLocale('fr');
		expect(t('app.tagline')).toBe('Laissez respirer votre PC.');
		setLocale('en');
		expect(getLocale()).toBe('en');
		expect(t('app.tagline')).toBe('Let your PC breathe.');
	});
});
