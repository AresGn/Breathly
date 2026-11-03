<script lang="ts">
	import { t } from '#lib/i18n/i18n.svelte.js';
	import Icon from './Icon.svelte';

	let {
		explanation,
		title,
		onretry
	}: { explanation: string; title?: string; onretry?: () => void } = $props();
</script>

<div class="banner banner--warn" role="alert">
	<Icon name="alert" />
	<div class="grow">
		<span class="banner-title">{title ?? t('component.state.error.title')}</span>
		{explanation}
	</div>
	{#if onretry}
		<button type="button" class="btn btn--secondary btn--sm" onclick={onretry}>
			{t('component.state.error.retry')}
		</button>
	{/if}
</div>
