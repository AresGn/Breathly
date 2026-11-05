<script lang="ts">
	import { t } from '#lib/i18n/i18n.svelte.js';
</script>

<div class="card stack" role="status" aria-busy="true" aria-label={t('component.state.loading')}>
	<div class="skel skel--lg" style="width: 55%"></div>
	<div class="skel" style="height: 10px; border-radius: 999px"></div>
	<div class="skel" style="width: 40%"></div>
</div>
