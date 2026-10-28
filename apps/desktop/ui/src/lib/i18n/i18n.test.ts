import { afterEach, describe, expect, it } from 'vitest';
import { formatBytes } from './format.js';
import { getLocale, resetLocale, setLocale, t } from './i18n.svelte.js';
import { pickLocale } from './system-locale.js';

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

describe('pluriels et valeurs', () => {
	it('accorde selon la langue (0 est singulier en français)', () => {
		setLocale('fr');
		expect(t('home.filesFound', { count: 0 })).toBe('0 fichier trouvé');
		expect(t('home.filesFound', { count: 1 })).toBe('1 fichier trouvé');
		expect(t('home.filesFound', { count: 2 })).toBe('2 fichiers trouvés');
		setLocale('en');
		expect(t('home.filesFound', { count: 0 })).toBe('0 files found');
		expect(t('home.filesFound', { count: 1 })).toBe('1 file found');
	});

	it('formate les grands nombres selon la langue', () => {
		setLocale('en');
		expect(t('home.filesFound', { count: 1234 })).toBe('1,234 files found');
	});

	it('insère une taille avec son unité', () => {
		setLocale('fr');
		expect(plain(t('home.spaceFreed', { size: 2_000_000_000 }))).toBe('2 Go libérés');
		setLocale('en');
		expect(plain(t('home.spaceFreed', { size: 2_000_000_000 }))).toBe('2 GB freed');
	});
});

describe('changement de langue à chaud', () => {
	it('change les textes sans redémarrer', () => {
		setLocale('fr');
		expect(t('home.analyze')).toBe('Analyser');
		setLocale('en');
		expect(getLocale()).toBe('en');
		expect(t('home.analyze')).toBe('Analyze');
	});
});
