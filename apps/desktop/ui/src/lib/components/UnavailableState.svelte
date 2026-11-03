<script lang="ts">
	import { t } from '#lib/i18n/i18n.svelte.js';
	import Icon from './Icon.svelte';

	let { label, explanation }: { label: string; explanation?: string } = $props();
</script>

<div class="card stack" role="status">
	<div class="h3">{label}</div>
	<div>
		<span class="status status--na">
			<Icon name="info" />
			{t('component.state.unavailable')}
		</span>
	</div>
	<div class="small muted">{explanation ?? t('component.state.unavailable.description')}</div>
</div>
