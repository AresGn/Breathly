<script lang="ts">
	import { t } from '#lib/i18n/i18n.svelte.js';

	let { title, description }: { title?: string; description?: string } = $props();
</script>

<div class="component-state" role="status">
	<p class="component-state__title">{title ?? t('component.state.empty.title')}</p>
	<p class="component-state__description">
		{description ?? t('component.state.empty.description')}
	</p>
</div>
