<script lang="ts">
	import { ICON_PATHS, type IconName } from './icons.js';

	let { name, isLarge = false }: { name: IconName; isLarge?: boolean } = $props();
</script>

<svg
	class="ic"
	class:ic--lg={isLarge}
	viewBox="0 0 24 24"
	fill="none"
	stroke="currentColor"
	aria-hidden="true"
>
	{#each ICON_PATHS[name] as d (d)}
		<path {d} />
	{/each}
</svg>
