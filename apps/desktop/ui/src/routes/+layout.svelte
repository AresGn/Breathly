<script lang="ts">
	import '#lib/styles/breathly.css';
	import { page } from '$app/state';
	import Sidebar from '#lib/components/Sidebar.svelte';
	import { startThemeSync } from '#lib/theme/theme.svelte.js';
	import { onDestroy, type Snippet } from 'svelte';

	let { children }: { children: Snippet } = $props();

	// Appliqué dès l'initialisation, avant le premier affichage, pour éviter un flash de thème clair.
	onDestroy(startThemeSync());
</script>

<div class="app">
	<Sidebar pathname={page.url.pathname} />
	{@render children()}
</div>

<style>
	.app {
		display: flex;
		min-height: 100vh;
	}

	:global(body) {
		margin: 0;
		background: var(--color-bg);
		color: var(--color-text);
		font-family: var(--font-body);
		font-size: var(--text-md);
		line-height: var(--leading-body);
	}

	/* Aucune animation si l'utilisateur demande moins de mouvement (WCAG 2.3.3). */
	@media (prefers-reduced-motion: reduce) {
		:global(*),
		:global(*::before),
		:global(*::after) {
			animation: none !important;
			transition: none !important;
			scroll-behavior: auto !important;
		}
	}
</style>
