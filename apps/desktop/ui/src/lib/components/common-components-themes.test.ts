import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { cleanup, render } from '@testing-library/svelte';
import { afterEach, describe, expect, it } from 'vitest';
import EmptyState from './EmptyState.svelte';
import ErrorState from './ErrorState.svelte';
import LoadingState from './LoadingState.svelte';
import RiskLevelBadge from './RiskLevelBadge.svelte';
import UnavailableState from './UnavailableState.svelte';

type Rendering = readonly [name: string, renderIt: () => { container: HTMLElement }];

const THEMES = ['light', 'dark'] as const;
const RENDERINGS: readonly Rendering[] = [
	['RiskLevelBadge (safe)', () => render(RiskLevelBadge, { props: { level: 'safe' } })],
	['RiskLevelBadge (review)', () => render(RiskLevelBadge, { props: { level: 'review' } })],
	['RiskLevelBadge (personal)', () => render(RiskLevelBadge, { props: { level: 'personal' } })],
	['LoadingState', () => render(LoadingState)],
	['EmptyState', () => render(EmptyState)],
	['ErrorState', () => render(ErrorState, { props: { explanation: 'x', onretry: () => {} } })],
	['UnavailableState', () => render(UnavailableState, { props: { label: 'x' } })]
];

const css = readFileSync(join(import.meta.dirname, '../styles/breathly.css'), 'utf8').replace(
	/\/\*[\s\S]*?\*\//g,
	''
);
const rules = [...css.matchAll(/([^{}]+)\{([^{}]*)\}/g)].map(([, selector, body]) => ({
	selector: selector?.trim() ?? '',
	body: body ?? ''
}));

function definedTokens(theme: (typeof THEMES)[number]): ReadonlySet<string> {
	const tokens = rules
		.filter(({ selector }) => selector === ':root' || selector === `[data-theme='${theme}']`)
		.flatMap(({ body }) => [...body.matchAll(/(--[\w-]+)\s*:/g)].map(([, token]) => token ?? ''));
	return new Set(tokens);
}

function usedTokens(classNames: ReadonlySet<string>): ReadonlySet<string> {
	const isUsed = (selector: string) =>
		[...classNames].some((name) => new RegExp(`\\.${name}(?![\\w-])`).test(selector));
	const tokens = rules
		.filter(({ selector }) => isUsed(selector))
		.flatMap(({ body }) => [...body.matchAll(/var\((--[\w-]+)/g)].map(([, token]) => token ?? ''));
	return new Set(tokens);
}

afterEach(cleanup);

describe('jetons des composants communs dans les deux thèmes', () => {
	it.each(RENDERINGS)('%s : chaque jeton utilisé existe en clair et en sombre', (_, renderIt) => {
		const { container } = renderIt();
		const classNames = new Set(
			[...container.querySelectorAll('[class]')].flatMap((element) => [...element.classList])
		);
		const used = usedTokens(classNames);

		expect(used.size).toBeGreaterThan(0);
		for (const theme of THEMES) {
			const defined = definedTokens(theme);
			expect([...used].filter((token) => !defined.has(token))).toEqual([]);
		}
	});

	it('les jetons de couleur de niveau de risque diffèrent entre les deux thèmes', () => {
		const valueOf = (theme: (typeof THEMES)[number], token: string) =>
			rules
				.filter(({ selector }) => selector === `[data-theme='${theme}']`)
				.map(({ body }) => new RegExp(`${token}\\s*:\\s*([^;]+);`).exec(body)?.[1])
				.find((value) => value !== undefined);

		for (const token of ['--risk-safe-bg', '--risk-verify-bg', '--risk-personal-bg']) {
			expect(valueOf('light', token)).toBeDefined();
			expect(valueOf('light', token)).not.toBe(valueOf('dark', token));
		}
	});
});
