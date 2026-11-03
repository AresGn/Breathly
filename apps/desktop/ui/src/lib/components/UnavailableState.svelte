<script lang="ts">
	import { t } from '#lib/i18n/i18n.svelte.js';
	import Icon from './Icon.svelte';

	let { label, explanation }: { label: string; explanation?: string } = $props();
</script>

<div class="component-state" role="status">
	<p class="component-state__title">{label}</p>
	<span class="status status--na">
		<Icon name="info" />
		{t('component.state.unavailable')}
	</span>
	<p class="component-state__description">
		{explanation ?? t('component.state.unavailable.description')}
	</p>
</div>
