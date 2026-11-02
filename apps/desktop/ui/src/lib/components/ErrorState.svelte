<script lang="ts">
	import { t } from '#lib/i18n/i18n.svelte.js';

	let { explanation, title }: { explanation: string; title?: string } = $props();
</script>

<div class="component-state" role="alert">
	<span class="status status--alert">
		<svg class="ic" viewBox="0 0 24 24" fill="none" stroke="currentColor" aria-hidden="true">
			<path d="M12 4 2.8 19.5h18.4z" />
			<path d="M12 10v4.5M12 17.2h.01" />
		</svg>
		{title ?? t('component.state.error.title')}
	</span>
	<p class="component-state__description">{explanation}</p>
</div>
