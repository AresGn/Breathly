<script lang="ts">
	import { t } from '#lib/i18n/i18n.svelte.js';
	import { SCREENS, type ScreenId } from '#lib/navigation/screens.js';

	let { id }: { id: ScreenId } = $props();

	let heading: HTMLHeadingElement | undefined = $state();

	// Chaque route monte sa propre page : le focus va au titre à chaque changement d'écran,
	// pour que le lecteur d'écran annonce le nouvel écran et que le clavier reparte de son début.
	$effect(() => heading?.focus());
</script>

<main>
	<h1 tabindex="-1" bind:this={heading}>{t(SCREENS[id].titleKey)}</h1>
</main>

<style>
	main {
		flex: 1;
		min-width: 0;
		padding: var(--space-6);
	}

	h1 {
		margin: 0;
		font-family: var(--font-display);
		font-size: var(--text-2xl);
		line-height: var(--leading-tight);
	}

	h1:focus {
		outline: none;
	}
</style>
