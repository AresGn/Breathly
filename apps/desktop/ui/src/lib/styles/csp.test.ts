import { readdirSync, readFileSync } from 'node:fs';
import { relative, resolve } from 'node:path';
import { describe, expect, it } from 'vitest';

const root = resolve(import.meta.dirname, '../../..');
const buildDirectory = resolve(root, 'build');
const remoteResource =
	/(?:url\(\s*['"]?|@import\s+(?:url\(\s*)?['"]?|<(?:link|script|img|source)\b[^>]*?\b(?:href|src)\s*=\s*['"])(?:https?:)?\/\//i;
const sourceSheets = import.meta.glob<string>('/src/**/*.{css,svelte}', {
	query: '?raw',
	import: 'default',
	eager: true
});

function filesWithExtension(directory: string, extension: string): string[] {
	return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
		const path = resolve(directory, entry.name);
		if (entry.isDirectory()) return filesWithExtension(path, extension);
		return entry.isFile() && path.endsWith(extension) ? [path] : [];
	});
}

function externalReferences(files: string[]): string[] {
	return files
		.filter((file) => remoteResource.test(readFileSync(file, 'utf8')))
		.map((file) => relative(buildDirectory, file));
}

describe('contenu distant', () => {
	it("aucune source CSS, Svelte ou HTML ne référence d'URL externe", () => {
		const sources = [
			...Object.entries(sourceSheets),
			['/src/app.html', readFileSync(resolve(root, 'src/app.html'), 'utf8')]
		];
		const offenders = sources
			.filter(([, content]) => remoteResource.test(content))
			.map(([path]) => path);
		expect(offenders).toEqual([]);
	});

	it('détecte un lien de feuille de style distant écrit dans un composant', () => {
		expect(
			remoteResource.test('<link rel="stylesheet" href="https://example.test/fonts.css">')
		).toBe(true);
	});

	it('aucune feuille CSS ou page HTML construite ne référence une URL externe', () => {
		const stylesheets = filesWithExtension(buildDirectory, '.css');
		const pages = filesWithExtension(buildDirectory, '.html');
		expect(stylesheets).not.toHaveLength(0);
		expect(pages).not.toHaveLength(0);
		expect(externalReferences([...stylesheets, ...pages])).toEqual([]);
	});
});

describe('politique de sécurité du contenu', () => {
	it("la CSP autorise les ressources embarquées et rien d'autre", () => {
		const conf = readFileSync(resolve(root, '../src-tauri/tauri.conf.json'), 'utf8');
		const csp = /"csp":\s*"([^"]+)"/.exec(conf)?.[1];
		expect(csp).toBeDefined();
		if (!csp) throw new Error('La CSP est absente de tauri.conf.json.');

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
