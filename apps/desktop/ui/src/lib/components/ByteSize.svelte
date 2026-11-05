<script lang="ts">
	import { t } from '#lib/i18n/i18n.svelte.js';

	let { bytes }: { bytes: number } = $props();
</script>

<span>{t('component.byteSize', { size: bytes })}</span>
