<script lang="ts">
	import { APP_NAME } from '#lib/brand.js';
</script>

<main>
	<h1>{APP_NAME}</h1>
</main>

<style>
	main {
		display: grid;
		place-items: center;
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
