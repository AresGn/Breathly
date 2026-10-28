import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { getTheme, setThemePreference, startThemeSync } from './theme.svelte.js';
import { DARK_QUERY, resolveTheme } from './theme.js';

class FakeMediaQuery extends EventTarget implements MediaQueryList {
	media = DARK_QUERY;
	onchange = null;
	matches: boolean;
	constructor(matches: boolean) {
		super();
		this.matches = matches;
	}
	addListener(): void {}
	removeListener(): void {}
	setDark(isDark: boolean): void {
		this.matches = isDark;
		this.dispatchEvent(Object.assign(new Event('change'), { matches: isDark }));
	}
}

let media = new FakeMediaQuery(false);
let stop = (): void => {};
const applied = (): string | undefined => document.documentElement.dataset['theme'];

beforeEach(() => {
	media = new FakeMediaQuery(false);
	Object.defineProperty(window, 'matchMedia', {
		configurable: true,
		writable: true,
		value: vi.fn(() => media)
	});
});

afterEach(() => {
	stop();
	setThemePreference('system');
});

describe('choix du thème', () => {
	it('clair et sombre ignorent l’OS', () => {
		expect(resolveTheme('light', true)).toBe('light');
		expect(resolveTheme('dark', false)).toBe('dark');
	});

	it('système prend le thème de l’OS', () => {
		expect(resolveTheme('system', true)).toBe('dark');
		expect(resolveTheme('system', false)).toBe('light');
	});
});

describe('application sur le document', () => {
	it('suit l’OS au changement quand le choix est « système »', () => {
		stop = startThemeSync();
		expect(applied()).toBe('light');
		media.setDark(true);
		expect(applied()).toBe('dark');
		expect(getTheme()).toBe('dark');
		media.setDark(false);
		expect(applied()).toBe('light');
	});

	it('ne suit pas l’OS quand le thème est fixé', () => {
		stop = startThemeSync();
		setThemePreference('light');
		media.setDark(true);
		expect(applied()).toBe('light');
		setThemePreference('dark');
		expect(applied()).toBe('dark');
	});

	it('arrête le suivi', () => {
		stop = startThemeSync();
		stop();
		media.setDark(true);
		expect(applied()).toBe('light');
	});
});
