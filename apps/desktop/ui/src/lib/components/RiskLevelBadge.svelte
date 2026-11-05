<script lang="ts">
	import { t } from '#lib/i18n/i18n.svelte.js';
	import Icon from './Icon.svelte';
	import { RISK_PRESENTATIONS, type RiskLevel } from './risk-level.js';

	let { level }: { level: RiskLevel } = $props();
	const risk = $derived(RISK_PRESENTATIONS[level]);
</script>

<span class="risk {risk.className}">
	<Icon name={risk.icon} />
	{t(risk.label)}
</span>
