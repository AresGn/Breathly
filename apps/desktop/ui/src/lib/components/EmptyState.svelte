<script lang="ts">
	import { t } from '#lib/i18n/i18n.svelte.js';
	import Icon from './Icon.svelte';

	let { title, description }: { title?: string; description?: string } = $props();
</script>

<div class="card" role="status">
	<div class="empty">
		<Icon name="check" isLarge />
		<div class="h3">{title ?? t('component.state.empty.title')}</div>
		<div class="small muted">{description ?? t('component.state.empty.description')}</div>
	</div>
</div>
