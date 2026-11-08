import { existsSync, readFileSync, statSync } from 'node:fs';
import { resolve } from 'node:path';
import { describe, expect, it } from 'vitest';

const root = resolve(import.meta.dirname, '../../..');
const fontsCss = readFileSync(resolve(root, 'src/lib/styles/fonts.css'), 'utf8');
const sheets = import.meta.glob<string>('/src/**/*.{css,svelte}', {
	query: '?raw',
	import: 'default',
	eager: true
});
const remoteUrl = /(?:url\(\s*['"]?|@import\s+['"]?)(?:https?:)?\/\//i;
const licenses = ['bricolagegrotesque', 'hankengrotesk', 'dmmono'];
const maxFontsBytes = 150 * 1024;

function embeddedFiles(): string[] {
	return [...fontsCss.matchAll(/url\('(\/fonts\/[^']+)'\)/g)].map((match) => match[1] ?? '');
}

describe('polices embarquées', () => {
	it('aucune feuille de style ne charge une ressource distante', () => {
		const offenders = Object.entries(sheets)
			.filter(([, content]) => remoteUrl.test(content))
			.map(([path]) => path);
		expect(offenders).toEqual([]);
	});

	it('la page de départ ne référence aucune police distante', () => {
		const html = readFileSync(resolve(root, 'src/app.html'), 'utf8');
		expect(html).not.toMatch(/fonts\.(googleapis|gstatic)\.com|https?:\/\//);
	});

	it('chaque police déclarée existe en WOFF2 dans static/fonts', () => {
		const files = embeddedFiles();
		expect(files).toHaveLength(4);
		for (const file of files) {
			expect(existsSync(resolve(root, 'static', file.slice(1)))).toBe(true);
		}
	});

	it('chaque famille a son fichier de licence OFL', () => {
		for (const name of licenses) {
			const text = readFileSync(resolve(root, `static/fonts/OFL-${name}.txt`), 'utf8');
			expect(text).toContain('SIL OPEN FONT LICENSE Version 1.1');
		}
	});

	it('le poids total des polices reste sous 150 Kio', () => {
		const total = embeddedFiles().reduce(
			(sum, file) => sum + statSync(resolve(root, 'static', file.slice(1))).size,
			0
		);
		expect(total).toBeLessThan(maxFontsBytes);
	});

	it("la CSP de la fenêtre autorise les polices embarquées et rien d'autre", () => {
		const conf = readFileSync(resolve(root, '../src-tauri/tauri.conf.json'), 'utf8');
		const csp = /"csp":\s*"([^"]+)"/.exec(conf)?.[1] ?? '';
		const directives = new Map(
			csp.split(';').map((part) => {
				const [name = '', ...values] = part.trim().split(/\s+/);
				return [name, values];
			})
		);
		expect(directives.get('font-src')).toEqual(["'self'"]);
		expect(directives.get('style-src')).toEqual(["'self'"]);
		expect(directives.get('default-src')).toEqual(["'self'"]);
	});
});
