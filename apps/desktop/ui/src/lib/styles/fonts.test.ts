import { existsSync, readdirSync, readFileSync, statSync } from 'node:fs';
import { resolve } from 'node:path';
import { describe, expect, it } from 'vitest';

const root = resolve(import.meta.dirname, '../../..');
const fontsDirectory = resolve(root, 'static/fonts');
const fontsCss = readFileSync(resolve(root, 'src/lib/styles/fonts.css'), 'utf8');
const maxFontsBytes = 150 * 1024;

function staticPath(file: string): string {
	return resolve(root, 'static', file.slice(1));
}

function fontFamilies(): string[] {
	return [
		...new Set(
			[...fontsCss.matchAll(/font-family:\s*'([^']+)'/g)].map((match) => {
				const family = match[1];
				if (!family) throw new Error('Une famille de police est mal formée.');
				return family;
			})
		)
	];
}

function embeddedFiles(): string[] {
	return [...fontsCss.matchAll(/url\('(\/fonts\/[^']+)'\)/g)].map((match) => {
		const path = match[1];
		if (!path) throw new Error('Une URL de police embarquée est mal formée.');
		return path;
	});
}

describe('polices embarquées', () => {
	it('chaque police déclarée existe en WOFF2 dans static/fonts', () => {
		const files = embeddedFiles();
		expect(files).not.toHaveLength(0);
		for (const file of files) {
			expect(existsSync(staticPath(file))).toBe(true);
		}
	});

	it('chaque famille a son fichier de licence OFL', () => {
		const expectedLicenses = fontFamilies()
			.map((family) => `OFL-${family.toLowerCase().replaceAll(/\s+/g, '')}.txt`)
			.sort();
		const licenseFiles = readdirSync(fontsDirectory)
			.filter((file) => file.startsWith('OFL-') && file.endsWith('.txt'))
			.sort();
		expect(licenseFiles).toEqual(expectedLicenses);

		for (const file of licenseFiles) {
			const text = readFileSync(resolve(fontsDirectory, file), 'utf8');
			expect(text).toContain('SIL OPEN FONT LICENSE Version 1.1');
		}
	});

	it('le poids total des polices reste sous 150 Kio', () => {
		const total = embeddedFiles().reduce((sum, file) => sum + statSync(staticPath(file)).size, 0);
		expect(total).toBeLessThan(maxFontsBytes);
	});
});
