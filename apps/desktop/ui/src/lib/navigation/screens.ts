import type { RouteId } from '$app/types';
import type { MessageKey } from '#lib/i18n/catalog.js';

// Inventaire des écrans de la V0.1 (design 0003 §5), hors T1 qui est de la V0.3.
export type ScreenId =
	| 'H1'
	| 'H2'
	| 'D1'
	| 'D2'
	| 'D3'
	| 'S1'
	| 'S2'
	| 'S3'
	| 'N1'
	| 'N2'
	| 'N3'
	| 'N4'
	| 'N5'
	| 'Y1'
	| 'Y2'
	| 'Y3'
	| 'R1'
	| 'R2'
	| 'R3';

// Entrée de la barre latérale à laquelle un écran se rattache.
export type Section = 'home' | 'diagnostic' | 'disk' | 'history' | 'settings';

export type Screen = {
	readonly id: ScreenId;
	readonly path: RouteId;
	readonly section: Section;
	readonly titleKey: MessageKey;
};

export const SCREENS: Readonly<Record<ScreenId, Screen>> = {
	H1: { id: 'H1', path: '/', section: 'home', titleKey: 'screen.H1' },
	H2: { id: 'H2', path: '/analyse', section: 'home', titleKey: 'screen.H2' },
	D1: { id: 'D1', path: '/diagnostic', section: 'diagnostic', titleKey: 'screen.D1' },
	D2: { id: 'D2', path: '/diagnostic/chauffe', section: 'diagnostic', titleKey: 'screen.D2' },
	D3: { id: 'D3', path: '/diagnostic/reglage', section: 'diagnostic', titleKey: 'screen.D3' },
	S1: { id: 'S1', path: '/espace', section: 'disk', titleKey: 'screen.S1' },
	S2: { id: 'S2', path: '/espace/elements', section: 'disk', titleKey: 'screen.S2' },
	S3: { id: 'S3', path: '/espace/gros-fichiers', section: 'disk', titleKey: 'screen.S3' },
	N1: { id: 'N1', path: '/nettoyage/confirmation', section: 'disk', titleKey: 'screen.N1' },
	N2: { id: 'N2', path: '/nettoyage/simulation', section: 'disk', titleKey: 'screen.N2' },
	N3: { id: 'N3', path: '/nettoyage/en-cours', section: 'disk', titleKey: 'screen.N3' },
	N4: { id: 'N4', path: '/nettoyage/resume', section: 'disk', titleKey: 'screen.N4' },
	N5: { id: 'N5', path: '/nettoyage/corbeille', section: 'disk', titleKey: 'screen.N5' },
	Y1: { id: 'Y1', path: '/historique', section: 'history', titleKey: 'screen.Y1' },
	Y2: { id: 'Y2', path: '/historique/annulation', section: 'history', titleKey: 'screen.Y2' },
	Y3: { id: 'Y3', path: '/historique/effacer', section: 'history', titleKey: 'screen.Y3' },
	R1: { id: 'R1', path: '/reglages', section: 'settings', titleKey: 'screen.R1' },
	R2: { id: 'R2', path: '/reglages/seuils', section: 'settings', titleKey: 'screen.R2' },
	R3: { id: 'R3', path: '/reglages/exclusions', section: 'settings', titleKey: 'screen.R3' }
};

export type NavEntry = {
	readonly section: Section;
	readonly path: RouteId;
	readonly labelKey: MessageKey;
};

// Ordre fixé par le design 0003 §3.
export const NAV_ENTRIES: readonly NavEntry[] = [
	{ section: 'home', path: SCREENS.H1.path, labelKey: 'nav.home' },
	{ section: 'diagnostic', path: SCREENS.D1.path, labelKey: 'nav.diagnostic' },
	{ section: 'disk', path: SCREENS.S1.path, labelKey: 'nav.disk' },
	{ section: 'history', path: SCREENS.Y1.path, labelKey: 'nav.history' },
	{ section: 'settings', path: SCREENS.R1.path, labelKey: 'nav.settings' }
];

export function normalizePath(pathname: string): string {
	return pathname.length > 1 && pathname.endsWith('/') ? pathname.slice(0, -1) : pathname;
}

export function sectionOfPath(pathname: string): Section | undefined {
	const path = normalizePath(pathname);
	return Object.values(SCREENS).find((screen) => screen.path === path)?.section;
}
