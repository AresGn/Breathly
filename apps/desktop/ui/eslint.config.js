import prettier from 'eslint-config-prettier';
import path from 'node:path';
import js from '@eslint/js';
import svelte from 'eslint-plugin-svelte';
import { defineConfig, includeIgnoreFile } from 'eslint/config';
import globals from 'globals';
import ts from 'typescript-eslint';

const gitignorePath = path.resolve(import.meta.dirname, '.gitignore');

export default defineConfig(
	includeIgnoreFile(gitignorePath),
	js.configs.recommended,
	ts.configs.recommended,
	svelte.configs.recommended,
	prettier,
	svelte.configs.prettier,
	{
		languageOptions: { globals: { ...globals.browser, ...globals.node } },
		rules: {
			'no-undef': 'off',
			// Taille et complexité (AGENTS.md, CODING.md §2)
			complexity: ['error', 10],
			'max-depth': ['error', 3],
			'max-params': ['error', 3],
			'max-lines-per-function': ['error', { max: 40, skipBlankLines: true, skipComments: true }],
			'max-lines': ['error', { max: 250, skipBlankLines: true, skipComments: true }],
			// Types (CODING.md §4)
			'@typescript-eslint/ban-ts-comment': [
				'error',
				{
					'ts-ignore': true,
					'ts-expect-error': 'allow-with-description',
					minimumDescriptionLength: 10
				}
			],
			// Journaux (SEC9)
			'no-console': 'error',
			'no-eval': 'error',
			'no-implied-eval': 'error',
			'svelte/no-at-html-tags': 'error'
			// Suppressions de contrôle : une raison écrite (CONTRIBUTING règle 6)
		}
	},
	{
		files: ['**/*.ts', '**/*.svelte', '**/*.svelte.ts'],
		languageOptions: {
			parserOptions: {
				projectService: true,
				extraFileExtensions: ['.svelte'],
				parser: ts.parser
			}
		},
		rules: {
			// États en union fermée : tous les cas, vérifié par le typage
			'@typescript-eslint/switch-exhaustiveness-check': [
				'error',
				{ considerDefaultExhaustiveForUnions: false, requireDefaultForNonUnion: false }
			],
			'@typescript-eslint/no-explicit-any': 'error',
			'@typescript-eslint/no-unsafe-assignment': 'error',
			'@typescript-eslint/no-unsafe-argument': 'error',
			'@typescript-eslint/no-unsafe-member-access': 'error',
			'@typescript-eslint/no-unsafe-call': 'error',
			'@typescript-eslint/no-unsafe-return': 'error',
			'@typescript-eslint/consistent-type-assertions': ['error', { assertionStyle: 'never' }]
		}
	},
	{
		// Seul src/lib/api/ appelle les commandes Tauri (CODING.md §1)
		files: ['src/**/*.{ts,svelte}'],
		ignores: ['src/lib/api/**'],
		rules: {
			'no-restricted-imports': [
				'error',
				{
					patterns: [
						{
							group: ['@tauri-apps/*'],
							message: 'Seul src/lib/api/ appelle Tauri : passez par ce module.'
						}
					]
				}
			]
		}
	},
	{
		files: ['**/*.test.ts'],
		rules: {
			'max-lines-per-function': 'off',
			// Aucun test sauté, isolé ou laissé à faire (CONTRIBUTING règle 6)
			'no-restricted-properties': [
				'error',
				...['it', 'test', 'describe'].flatMap((object) =>
					['skip', 'skipIf', 'only', 'todo', 'fails'].map((property) => ({
						object,
						property,
						message: 'Un test ne se saute pas, ne s’isole pas et ne se reporte pas.'
					}))
				)
			]
		}
	}
);
