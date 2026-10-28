import { readdirSync, readFileSync, statSync } from 'node:fs';
import { join } from 'node:path';
import { describe, expect, it } from 'vitest';
import { CATALOGS, LOCALES } from './catalog.js';

const SRC = join(import.meta.dirname, '../..');

function sourceFiles(dir: string): string[] {
	return readdirSync(dir).flatMap((name) => {
		const path = join(dir, name);
		if (statSync(path).isDirectory()) return sourceFiles(path);
		return /\.(ts|svelte)$/.test(name) ? [path] : [];
	});
}

function isReferenced(key: string, sources: readonly string[]): boolean {
	return sources.some((source) => source.includes(`'${key}'`));
}

describe('clés de traduction (US-002.3)', () => {
	it('chaque clé du français existe en anglais, et inversement', () => {
		const [first, ...others] = LOCALES.map((locale) => Object.keys(CATALOGS[locale]).sort());
		for (const keys of others) expect(keys).toEqual(first);
	});

	it('chaque clé a le même type de message dans les deux langues', () => {
		const english = new Map(Object.entries(CATALOGS.en));
		for (const [key, message] of Object.entries(CATALOGS.fr)) {
			expect(typeof english.get(key)).toBe(typeof message);
		}
	});

	it('aucune clé n’est orpheline : chacune est utilisée dans src/', () => {
		const sources = sourceFiles(SRC)
			.filter((path) => !path.endsWith('cles.test.ts'))
			.map((path) => readFileSync(path, 'utf8'));
		const orphans = Object.keys(CATALOGS.fr).filter((key) => !isReferenced(key, sources));
		expect(orphans).toEqual([]);
	});

	it('détecte une clé orpheline', () => {
		expect(isReferenced('home.absente', ["t('home.analyze')"])).toBe(false);
		expect(isReferenced('home.analyze', ["t('home.analyze')"])).toBe(true);
	});
});
