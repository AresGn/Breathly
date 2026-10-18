import { describe, expect, it } from 'vitest';

const sources = import.meta.glob<string>('/src/**/*.{svelte,css,ts}', {
	query: '?raw',
	import: 'default',
	eager: true
});
const colorLiteral = /#[0-9a-fA-F]{3,8}\b|\b(?:rgb|rgba|hsl|hsla|oklch)\(/;

describe('jetons de design', () => {
	it('aucune couleur en dur hors de breathly.css', () => {
		const offenders = Object.entries(sources)
			.filter(([path]) => !path.endsWith('/styles/breathly.css') && !path.endsWith('.test.ts'))
			.filter(([, content]) => colorLiteral.test(content))
			.map(([path]) => path);
		expect(offenders).toEqual([]);
	});
});
