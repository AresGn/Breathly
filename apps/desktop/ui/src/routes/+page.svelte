<script lang="ts">
	import { APP_NAME } from '#lib/brand.js';
	import { t } from '#lib/i18n/i18n.svelte.js';
</script>

<main>
	<h1>{APP_NAME}</h1>
	<p>{t('app.tagline')}</p>
</main>

<style>
	main {
		display: grid;
		place-content: center;
		justify-items: center;
		min-height: 100vh;
	}

	h1 {
		margin: 0;
		font-family: var(--font-display);
		font-size: var(--text-3xl);
		line-height: var(--leading-tight);
		color: var(--color-accent);
	}
</style>
