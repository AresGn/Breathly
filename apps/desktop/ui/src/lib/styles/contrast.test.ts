import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { describe, expect, it } from 'vitest';

type Pair = readonly [foreground: string, background: string];

const TEXT_RATIO = 4.5;
const GRAPHIC_RATIO = 3;

const THEMES = ['light', 'dark'] as const;

// Paires texte / fond réellement utilisées par les maquettes (WCAG 1.4.3).
const TEXT_PAIRS: readonly Pair[] = [
	['text', 'bg'],
	['text', 'surface'],
	['text', 'surface-2'],
	['text', 'sidebar'],
	['text-muted', 'bg'],
	['text-muted', 'surface'],
	['text-muted', 'surface-2'],
	['text-muted', 'sidebar'],
	['accent', 'bg'],
	['accent', 'surface'],
	['accent', 'accent-soft'],
	['on-accent', 'accent'],
	['on-accent', 'accent-hover'],
	['risk-safe-fg', 'risk-safe-bg'],
	['risk-verify-fg', 'risk-verify-bg'],
	['risk-personal-fg', 'risk-personal-bg'],
	['color-success-fg', 'color-success-bg'],
	['color-warn-fg', 'color-warn-bg'],
	['color-alert-fg', 'color-alert-bg'],
	['on-danger', 'danger-solid']
];

// Composants graphiques : bordures de champ, focus, icônes d'accent (WCAG 1.4.11).
const GRAPHIC_PAIRS: readonly Pair[] = [
	['border-strong', 'bg'],
	['border-strong', 'surface'],
	['focus', 'bg'],
	['focus', 'surface'],
	['accent', 'bg'],
	['accent', 'surface'],
	['danger-solid', 'surface']
];

const css = readFileSync(join(import.meta.dirname, 'breathly.css'), 'utf8');

function themeTokens(theme: string): Map<string, string> {
	const block =
		new RegExp(String.raw`\[data-theme='${theme}'\]\s*\{([^}]*)\}`).exec(css)?.[1] ?? '';
	const entries = [...block.matchAll(/--([\w-]+):\s*(#[0-9a-fA-F]{6})\s*;/g)];
	return new Map(entries.map(([, name = '', value = '']) => [name, value]));
}

function channel(hex: string, offset: number): number {
	const value = parseInt(hex.slice(offset, offset + 2), 16) / 255;
	return value <= 0.03928 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4;
}

function luminance(hex: string): number {
	const color = hex.slice(1);
	return 0.2126 * channel(color, 0) + 0.7152 * channel(color, 2) + 0.0722 * channel(color, 4);
}

export function contrast(first: string, second: string): number {
	const [light, dark] = [luminance(first), luminance(second)].sort((a, b) => b - a);
	return ((light ?? 0) + 0.05) / ((dark ?? 0) + 0.05);
}

function failures(theme: string, pairs: readonly Pair[], minimum: number): string[] {
	const tokens = themeTokens(theme);
	return pairs.flatMap(([foreground, background]) => {
		const fg = tokens.get(foreground) ?? tokens.get(`color-${foreground}`);
		const bg = tokens.get(background) ?? tokens.get(`color-${background}`);
		if (!fg || !bg) return [`${theme}: jeton absent (${foreground} / ${background})`];
		const ratio = contrast(fg, bg);
		return ratio >= minimum
			? []
			: [`${theme}: ${foreground} / ${background} = ${ratio.toFixed(2)}`];
	});
}

describe('contraste WCAG 2.2 AA (US-003.2)', () => {
	it('calcule le rapport de contraste', () => {
		expect(contrast('#000000', '#ffffff')).toBeCloseTo(21, 5);
		expect(contrast('#767676', '#ffffff')).toBeCloseTo(4.54, 2);
	});

	it.each(THEMES)('texte : au moins 4,5:1 (thème %s)', (theme) => {
		expect(failures(theme, TEXT_PAIRS, TEXT_RATIO)).toEqual([]);
	});

	it.each(THEMES)('composants graphiques : au moins 3:1 (thème %s)', (theme) => {
		expect(failures(theme, GRAPHIC_PAIRS, GRAPHIC_RATIO)).toEqual([]);
	});
});
