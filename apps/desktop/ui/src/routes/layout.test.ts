import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { describe, expect, it } from 'vitest';

describe('mouvement réduit', () => {
	it('la mise en page coupe animations et transitions sous prefers-reduced-motion', () => {
		const layout = readFileSync(join(import.meta.dirname, '+layout.svelte'), 'utf8');
		expect(layout).toMatch(/@media \(prefers-reduced-motion: reduce\)[^]*animation: none/);
		expect(layout).toMatch(/@media \(prefers-reduced-motion: reduce\)[^]*transition: none/);
	});
});
