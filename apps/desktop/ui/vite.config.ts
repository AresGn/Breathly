import { defineConfig } from 'vitest/config';
import adapter from '@sveltejs/adapter-static';
import { sveltekit } from '@sveltejs/kit/vite';
import { svelteTesting } from '@testing-library/svelte/vite';

export default defineConfig({
	// Tauri attend un port fixe (tauri.conf.json : devUrl) et ne doit pas masquer les erreurs Rust.
	clearScreen: false,
	server: { port: 5173, strictPort: true, host: '127.0.0.1' },
	plugins: [
		sveltekit({
			compilerOptions: {
				runes: ({ filename }) =>
					filename.split(/[/\\]/).includes('node_modules') ? undefined : true
			},
			adapter: adapter({ fallback: 'index.html' })
		}),
		svelteTesting()
	],
	test: {
		environment: 'jsdom',
		include: ['src/**/*.{test,spec}.{js,ts}'],
		setupFiles: ['./vitest-setup.ts'],
		expect: { requireAssertions: true }
	}
});
