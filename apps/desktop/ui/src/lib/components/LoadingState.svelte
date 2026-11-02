<script lang="ts">
	import { t } from '#lib/i18n/i18n.svelte.js';
</script>

<div class="component-state" role="status" aria-busy="true">
	<span class="status status--info">
		<svg class="ic" viewBox="0 0 24 24" fill="none" stroke="currentColor" aria-hidden="true">
			<circle cx="12" cy="12" r="9" />
			<path d="M12 11v5.5M12 7.8h.01" />
		</svg>
		{t('component.state.loading')}
	</span>
</div>
