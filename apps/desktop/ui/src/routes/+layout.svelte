<script lang="ts">
	import '#lib/styles/breathly.css';
	import { startThemeSync } from '#lib/theme/theme.svelte.js';
	import { onMount, type Snippet } from 'svelte';

	let { children }: { children: Snippet } = $props();

	onMount(startThemeSync);
</script>

{@render children()}

<style>
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
