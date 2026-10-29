import { describe, expect, it } from 'vitest';
import { CATALOGS, LOCALES } from '#lib/i18n/catalog.js';
import { NAV_ENTRIES, SCREENS, sectionOfPath } from './screens.js';

// Inventaire du design 0003 §5, hors T1 (V0.3). Écrit à la main : ajouter un écran au code
// sans l'ajouter ici (ou l'inverse) fait échouer le test.
const INVENTORY = [
	...['H1', 'H2'],
	...['D1', 'D2', 'D3'],
	...['S1', 'S2', 'S3'],
	...['N1', 'N2', 'N3', 'N4', 'N5'],
	...['Y1', 'Y2', 'Y3'],
	...['R1', 'R2', 'R3']
];

const routeFiles = Object.keys(import.meta.glob('/src/routes/**/+page.svelte'));

function routeFileOf(path: string): string {
	return `/src/routes${path === '/' ? '' : path}/+page.svelte`;
}

describe('routes des écrans (US-070.1)', () => {
	it('l’inventaire du code est celui du design 0003 §5', () => {
		expect(Object.keys(SCREENS).sort()).toEqual([...INVENTORY].sort());
	});

	it.each(INVENTORY)('l’écran %s a sa route', (id) => {
		const screen = Object.values(SCREENS).find((candidate) => candidate.id === id);
		expect(screen).toBeDefined();
		expect(routeFiles).toContain(routeFileOf(screen?.path ?? ''));
	});

	it('chaque route correspond à un écran de l’inventaire', () => {
		const expected = Object.values(SCREENS).map((screen) => routeFileOf(screen.path));
		expect(routeFiles.sort()).toEqual(expected.sort());
	});

	it('deux écrans n’ont jamais le même chemin', () => {
		const paths = Object.values(SCREENS).map((screen) => screen.path);
		expect(new Set(paths).size).toBe(paths.length);
	});

	it('chaque écran a un titre dans chaque langue', () => {
		for (const locale of LOCALES) {
			for (const screen of Object.values(SCREENS)) {
				expect(CATALOGS[locale][screen.titleKey]).toBeTruthy();
			}
		}
	});
});

describe('barre latérale (design 0003 §3)', () => {
	it('liste Accueil, Diagnostic, Espace disque, Historique, Réglages dans cet ordre', () => {
		expect(NAV_ENTRIES.map((entry) => entry.section)).toEqual([
			'home',
			'diagnostic',
			'disk',
			'history',
			'settings'
		]);
	});

	it('chaque entrée mène à un écran de sa section', () => {
		for (const entry of NAV_ENTRIES) expect(sectionOfPath(entry.path)).toBe(entry.section);
	});

	it('rattache les écrans de nettoyage à Espace disque', () => {
		expect(sectionOfPath(SCREENS.N4.path)).toBe('disk');
	});

	it('ignore la barre oblique finale et les chemins inconnus', () => {
		expect(sectionOfPath('/historique/')).toBe('history');
		expect(sectionOfPath('/inconnu')).toBeUndefined();
	});
});
